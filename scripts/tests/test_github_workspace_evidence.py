import base64
import importlib.util
import unittest
from pathlib import Path


SCRIPT = Path(__file__).resolve().parents[1] / "github_workspace_evidence.py"
SPEC = importlib.util.spec_from_file_location("github_workspace_evidence", SCRIPT)
MODULE = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
SPEC.loader.exec_module(MODULE)


class GitHubWorkspaceEvidenceTests(unittest.TestCase):
    def test_collects_matching_pr_and_stable_cirru(self):
        target = "0.15.8"
        deps = base64.b64encode(b"{} (:calcit-version |0.15.8)\n").decode()

        def api(endpoint, *, allow_not_found=False):
            responses = {
                "repos/org/app": {"archived": False, "default_branch": "main"},
                "repos/org/app/commits/main": {"sha": "1" * 40},
                "repos/org/app/contents/deps.cirru?ref=main": {"content": deps},
                "repos/org/app/releases/latest": {"tag_name": "v1.2.3"},
                "repos/org/app/pulls?state=open&per_page=100": [
                    {
                        "number": 7,
                        "html_url": "https://github.com/org/app/pull/7",
                        "head": {
                            "sha": "2" * 40,
                            "repo": {"full_name": "contributor/app"},
                        },
                    }
                ],
                f"repos/contributor/app/contents/deps.cirru?ref={'2' * 40}": {"content": deps},
                "repos/org/app/pulls/7": {
                    "html_url": "https://github.com/org/app/pull/7",
                    "mergeable": True,
                    "mergeable_state": "clean",
                },
                f"repos/contributor/app/commits/{'2' * 40}/check-runs": {
                    "check_runs": [{"status": "completed", "conclusion": "success"}]
                },
                f"repos/contributor/app/commits/{'2' * 40}/status": {
                    "state": "success",
                    "statuses": [],
                },
                "repos/org/app/pulls/7/reviews": [
                    {"submitted_at": "2026-09-17T12:00:00Z", "state": "APPROVED", "user": {"login": "reviewer"}}
                ],
            }
            self.assertIn(endpoint, responses)
            return responses[endpoint]

        project = MODULE.collect_repository(
            api, {"repository": "org/app", "deps-file": "deps.cirru"}, target
        )
        self.assertEqual(project["calcit-version"], target)
        self.assertEqual(project["latest-release"], "v1.2.3")
        self.assertEqual(project["pull-request"]["checks-state"], "passing")
        self.assertEqual(project["pull-request"]["review-state"], "approved")
        evidence = {
            "schema-version": "1",
            "observed-at": "2026-09-17T14:30:00Z",
            "projects": [project],
        }
        output = MODULE.format_cirru_evidence(evidence)
        self.assertIn("  :projects $ []\n    {}\n", output)
        self.assertIn("        :number 7\n", output)
        self.assertIn("        :checks-state |passing\n", output)

    def test_checks_and_reviews_report_blockers(self):
        commit = "3" * 40

        def api(endpoint, *, allow_not_found=False):
            if endpoint.endswith("/check-runs"):
                return {"check_runs": [{"status": "completed", "conclusion": "failure"}]}
            if endpoint.endswith("/status"):
                return {"state": "success", "statuses": []}
            if endpoint.endswith("/reviews"):
                return [
                    {"submitted_at": "2026-09-17T12:00:00Z", "state": "APPROVED", "user": {"login": "a"}},
                    {"submitted_at": "2026-09-17T13:00:00Z", "state": "CHANGES_REQUESTED", "user": {"login": "b"}},
                ]
            self.fail(endpoint)

        self.assertEqual(MODULE.checks_state(api, "org/app", commit), "failing")
        self.assertEqual(MODULE.review_state(api, "org/app", 7), "changes-requested")


if __name__ == "__main__":
    unittest.main()
