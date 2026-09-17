#!/usr/bin/env python3
"""Collect read-only GitHub evidence for a Caps workspace plan."""

from __future__ import annotations

import argparse
import base64
import json
import re
import subprocess
import sys
from concurrent.futures import ThreadPoolExecutor
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Callable
from urllib.parse import quote


CALCIT_VERSION_RE = re.compile(r":calcit-version\s+\|?([^\s)]+)")
SEMVER_RE = re.compile(
    r"v?\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?(?:\+[0-9A-Za-z.-]+)?$"
)
FAILED_CHECK_CONCLUSIONS = {
    "action_required",
    "cancelled",
    "failure",
    "stale",
    "startup_failure",
    "timed_out",
}


class ApiNotFound(Exception):
    pass


def command_json(command: list[str]) -> Any:
    result = subprocess.run(
        command,
        check=False,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )
    if result.returncode != 0:
        raise RuntimeError(
            f"command failed ({result.returncode}): {' '.join(command)}\n{result.stderr.strip()}"
        )
    try:
        return json.loads(result.stdout)
    except json.JSONDecodeError as error:
        raise RuntimeError(f"command returned invalid JSON: {' '.join(command)}: {error}") from error


def gh_api(gh: str, endpoint: str, *, allow_not_found: bool = False) -> Any:
    result = subprocess.run(
        [gh, "api", endpoint],
        check=False,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )
    if result.returncode != 0:
        if allow_not_found and ("HTTP 404" in result.stderr or "Not Found" in result.stderr):
            raise ApiNotFound(endpoint)
        raise RuntimeError(f"gh api {endpoint} failed: {result.stderr.strip()}")
    try:
        return json.loads(result.stdout)
    except json.JSONDecodeError as error:
        raise RuntimeError(f"gh api {endpoint} returned invalid JSON: {error}") from error


def remote_file(
    api: Callable[..., Any], repository: str, path: str, revision: str
) -> str | None:
    endpoint = (
        f"repos/{repository}/contents/{quote(path, safe='/')}?ref={quote(revision, safe='')}"
    )
    try:
        response = api(endpoint, allow_not_found=True)
    except ApiNotFound:
        return None
    content = response.get("content") if isinstance(response, dict) else None
    if not isinstance(content, str):
        return None
    try:
        return base64.b64decode(content).decode()
    except (ValueError, UnicodeDecodeError) as error:
        raise RuntimeError(f"invalid base64 content from {endpoint}: {error}") from error


def declared_calcit_version(content: str | None) -> str | None:
    if content is None:
        return None
    match = CALCIT_VERSION_RE.search(content)
    return match.group(1) if match else None


def checks_state(api: Callable[..., Any], repository: str, commit: str) -> str:
    check_runs = api(f"repos/{repository}/commits/{commit}/check-runs?per_page=100").get(
        "check_runs", []
    )
    combined = api(f"repos/{repository}/commits/{commit}/status")
    conclusions = [run.get("conclusion") for run in check_runs]
    if combined.get("state") == "failure" or any(
        conclusion in FAILED_CHECK_CONCLUSIONS for conclusion in conclusions
    ):
        return "failing"
    if combined.get("state") == "pending" or any(
        run.get("status") != "completed" or run.get("conclusion") is None for run in check_runs
    ):
        return "pending"
    if check_runs or combined.get("statuses"):
        return "passing"
    return "unknown"


def review_state(api: Callable[..., Any], repository: str, number: int) -> str:
    reviews = api(f"repos/{repository}/pulls/{number}/reviews?per_page=100")
    latest: dict[str, str] = {}
    for review in sorted(reviews, key=lambda item: item.get("submitted_at") or ""):
        user = review.get("user") or {}
        login = user.get("login")
        state = review.get("state")
        if isinstance(login, str) and state in {"APPROVED", "CHANGES_REQUESTED", "DISMISSED"}:
            latest[login] = state
    states = set(latest.values())
    if "CHANGES_REQUESTED" in states:
        return "changes-requested"
    if "APPROVED" in states:
        return "approved"
    return "pending"


def matching_pull_request(
    api: Callable[..., Any],
    repository: str,
    deps_file: str,
    target_calcit: str,
) -> dict[str, Any] | None:
    pulls = api(f"repos/{repository}/pulls?state=open&per_page=100")
    for pull in sorted(pulls, key=lambda item: item.get("number", 0)):
        head = pull.get("head") or {}
        commit = head.get("sha")
        if not isinstance(commit, str):
            continue
        head_repository = (head.get("repo") or {}).get("full_name") or repository
        content = remote_file(api, head_repository, deps_file, commit)
        if declared_calcit_version(content) != target_calcit:
            continue
        number = pull.get("number")
        if not isinstance(number, int):
            continue
        details = api(f"repos/{repository}/pulls/{number}")
        mergeable = details.get("mergeable")
        mergeable_state = details.get("mergeable_state")
        if mergeable is False or mergeable_state == "dirty":
            merge_state = "conflicted"
        elif mergeable is True:
            merge_state = "clean"
        else:
            merge_state = "unknown"
        return {
            "number": number,
            "url": details.get("html_url") or pull.get("html_url") or "",
            "head-commit": commit,
            "merge-state": merge_state,
            "checks-state": checks_state(api, head_repository, commit),
            "review-state": review_state(api, repository, number),
        }
    return None


def collect_repository(
    api: Callable[..., Any], project: dict[str, Any], target_calcit: str
) -> dict[str, Any]:
    repository = project["repository"]
    deps_file = project.get("deps-file") or "deps.cirru"
    metadata = api(f"repos/{repository}")
    default_branch = metadata.get("default_branch")
    if not isinstance(default_branch, str) or not default_branch:
        raise RuntimeError(f"GitHub returned no default branch for {repository}")
    commit_info = api(f"repos/{repository}/commits/{quote(default_branch, safe='')}")
    commit = commit_info.get("sha")
    if not isinstance(commit, str) or not commit:
        raise RuntimeError(f"GitHub returned no default-branch commit for {repository}")
    content = remote_file(api, repository, deps_file, default_branch)
    try:
        release = api(f"repos/{repository}/releases/latest", allow_not_found=True)
        latest_release = release.get("tag_name") if isinstance(release, dict) else None
        if not isinstance(latest_release, str) or not SEMVER_RE.fullmatch(latest_release):
            latest_release = None
    except ApiNotFound:
        latest_release = None
    return {
        "repository": repository,
        "archived": bool(metadata.get("archived")),
        "default-branch": default_branch,
        "default-branch-commit": commit,
        "calcit-version": declared_calcit_version(content),
        "latest-release": latest_release,
        "pull-request": matching_pull_request(api, repository, deps_file, target_calcit),
    }


def cirru_atom(value: Any) -> str:
    if value is None:
        return "nil"
    if value is True:
        return "true"
    if value is False:
        return "false"
    if isinstance(value, int):
        return str(value)
    if not isinstance(value, str):
        raise TypeError(f"unsupported Cirru EDN atom: {value!r}")
    if any(character.isspace() or character in "()" for character in value):
        raise ValueError(f"remote evidence string is not a safe Cirru token: {value!r}")
    return f"|{value}"


def format_cirru_evidence(evidence: dict[str, Any]) -> str:
    lines = [
        "{}",
        f"  :schema-version {cirru_atom(evidence['schema-version'])}",
        f"  :observed-at {cirru_atom(evidence['observed-at'])}",
        "  :projects $ []",
    ]
    project_fields = [
        "repository",
        "archived",
        "default-branch",
        "default-branch-commit",
        "calcit-version",
        "latest-release",
    ]
    pull_fields = [
        "number",
        "url",
        "head-commit",
        "merge-state",
        "checks-state",
        "review-state",
    ]
    for project in evidence["projects"]:
        lines.append("    {}")
        for field in project_fields:
            lines.append(f"      :{field} {cirru_atom(project.get(field))}")
        pull_request = project.get("pull-request")
        if pull_request is None:
            lines.append("      :pull-request nil")
        else:
            lines.append("      :pull-request $ {}")
            for field in pull_fields:
                lines.append(f"        :{field} {cirru_atom(pull_request.get(field))}")
    return "\n".join(lines) + "\n"


def load_plan(args: argparse.Namespace) -> dict[str, Any]:
    if args.plan_json:
        return json.loads(args.plan_json.read_text())
    return command_json(
        [args.caps, "tree", "--workspace", str(args.workspace), "--format", "json"]
    )


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    source = parser.add_mutually_exclusive_group(required=True)
    source.add_argument("--workspace", type=Path, help="Cirru EDN workspace inventory")
    source.add_argument("--plan-json", type=Path, help="existing explicit JSON plan")
    parser.add_argument("--caps", default="caps", help="Caps executable")
    parser.add_argument("--gh", default="gh", help="GitHub CLI executable")
    parser.add_argument("--repository", action="append", default=[], help="limit to owner/repo")
    parser.add_argument("--limit", type=int, default=0, help="limit repositories after sorting")
    parser.add_argument("--jobs", type=int, default=4, help="parallel GitHub readers")
    parser.add_argument("--format", choices=["cirru", "json"], default="cirru")
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()

    plan = load_plan(args)
    target_calcit = plan.get("target-calcit")
    if not isinstance(target_calcit, str):
        raise SystemExit("workspace plan has no string target-calcit")
    selected = [project for project in plan.get("projects", []) if isinstance(project, dict)]
    if args.repository:
        allowed = set(args.repository)
        selected = [project for project in selected if project.get("repository") in allowed]
    selected.sort(key=lambda project: project.get("repository", ""))
    if args.limit:
        selected = selected[: args.limit]
    api = lambda endpoint, **kwargs: gh_api(args.gh, endpoint, **kwargs)
    with ThreadPoolExecutor(max_workers=max(1, args.jobs)) as executor:
        projects = list(
            executor.map(lambda project: collect_repository(api, project, target_calcit), selected)
        )
    projects.sort(key=lambda project: project["repository"])
    evidence = {
        "schema-version": "1",
        "observed-at": datetime.now(timezone.utc)
        .replace(microsecond=0)
        .isoformat()
        .replace("+00:00", "Z"),
        "projects": projects,
    }
    content = (
        json.dumps(evidence, indent=2, ensure_ascii=False) + "\n"
        if args.format == "json"
        else format_cirru_evidence(evidence)
    )
    if args.output:
        args.output.write_text(content)
    else:
        sys.stdout.write(content)


if __name__ == "__main__":
    main()
