from __future__ import annotations

import re
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
ACCESSIBILITY_SCRIPT = (
    ROOT / "script" / "smoke_m16_macos_accessibility.sh"
).read_text(encoding="utf-8")


def shell_function(name: str) -> str:
    match = re.search(
        rf"(?ms)^{re.escape(name)}\(\) \{{\n(?P<body>.*?)^\}}$",
        ACCESSIBILITY_SCRIPT,
    )
    if match is None:
        raise AssertionError(f"missing shell function: {name}")
    return match.group("body")


def assert_in_order(test: unittest.TestCase, source: str, *needles: str) -> None:
    offset = 0
    for needle in needles:
        index = source.find(needle, offset)
        test.assertNotEqual(index, -1, f"missing or out-of-order shell step: {needle}")
        offset = index + len(needle)


class MacosSmokeContractTests(unittest.TestCase):
    def test_smokes_use_nonexistent_config_and_can_pin_an_exact_app(self) -> None:
        for name in ("smoke_m16_macos.sh", "smoke_m16_macos_accessibility.sh"):
            script = (ROOT / "script" / name).read_text(encoding="utf-8")
            self.assertIn("HONK300_APP", script)
            self.assertIn("HONK300_SKIP_BUILD", script)
            self.assertIn("TEMP_ROOT=", script)
            self.assertIn('CONFIG="${TEMP_ROOT}/config.toml"', script)
            self.assertNotIn("mktemp \"${TMPDIR:-/tmp}/honk300-m16-config", script)

    def test_accessibility_smoke_covers_wait_non_nag_and_same_identity_grant(self) -> None:
        script = ACCESSIBILITY_SCRIPT
        self.assertIn("accessibility-prompt-v1", script)
        self.assertIn("HONK300_ACCESSIBILITY_PHASE", script)
        self.assertIn("BUSY", script)
        self.assertIn("same signed app", script)

    def test_live_smoke_orders_prompt_non_nag_grant_and_revocation(self) -> None:
        live = shell_function("run_live_smoke")
        assert_in_order(
            self,
            live,
            "prepare_first_prompt_fixture",
            "start_runtime",
            "assert_denied_wait",
            "assert_prompt_marker",
            'require_operator_evidence "PROMPTED"',
            "stop_runtime",
            "start_runtime",
            "assert_denied_wait",
            "assert_prompt_marker_unchanged",
            'require_operator_evidence "NON_NAG"',
            "wait_for_live_grant",
            "assert_same_signed_app",
            'require_operator_evidence "FIRSTUX"',
            "wait_for_live_revocation",
            "assert_denied_wait",
            "assert_prompt_marker_unchanged",
            'require_operator_evidence "REVOCATION_QUIET"',
            "stop_runtime",
            "finalize_managed_fixture",
        )
        self.assertNotIn("package_macos_app.sh", live)

    def test_live_smoke_forces_scoped_first_prompt_and_checks_all_wait_controls(self) -> None:
        fixture = shell_function("prepare_first_prompt_fixture")
        self.assertIn("HONK300_SKIP_BUILD", fixture)
        self.assertIn("HONK300_RESET_PROMPT_MARKER", fixture)
        self.assertIn("remove_prompt_marker_safely", fixture)
        self.assertIn("HONK300_RESET_TCC", fixture)

        denied = shell_function("assert_denied_wait")
        self.assertIn('"${BIN}" reload', denied)
        self.assertIn("for action in wander mud nab meme note; do", denied)
        self.assertIn("BUSY", denied)

        grant = shell_function("wait_for_live_grant")
        revoke = shell_function("wait_for_live_revocation")
        for body, state in ((grant, "supported"), (revoke, "denied")):
            self.assertIn("monotonic_millis", body)
            self.assertIn("TRANSITION_DEADLINE_MS", body)
            self.assertIn(f'wait_for_accessibility_state "{state}"', body)

    def test_live_smoke_pins_identity_and_makes_destructive_cleanup_explicit(self) -> None:
        identity = shell_function("assert_same_signed_app")
        self.assertIn("codesign --verify --strict", identity)
        self.assertIn("shasum -a 256", identity)

        cleanup = shell_function("finalize_managed_fixture")
        self.assertIn("HONK300_FINAL_CLEANUP", cleanup)
        self.assertIn("purge-managed-install", cleanup)
        self.assertIn('"${BIN}" uninstall --purge', cleanup)
        self.assertIn("tccutil reset Accessibility", cleanup)
        self.assertIn("dev.emmetts.honk300", ACCESSIBILITY_SCRIPT)


if __name__ == "__main__":
    unittest.main()
