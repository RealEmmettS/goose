#!/usr/bin/env sh
set -eu

if [ "$(uname -s)" != "Darwin" ]; then
  echo "smoke_m16_macos_accessibility: must run on macOS" >&2
  exit 1
fi

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"
APP="${HONK300_APP:-${ROOT}/target/dist/macos-universal2/Honk300.app}"
BIN="${APP}/Contents/MacOS/honk300"
TEMP_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/honk300-m16-a11y.XXXXXX")"
CONFIG="${TEMP_ROOT}/config.toml"
STATUS="${TEMP_ROOT}/status.txt"
PHASE="${HONK300_ACCESSIBILITY_PHASE:-granted}"
BUNDLE_ID="dev.emmetts.honk300"
EXPECTED_APP="${HOME}/Applications/Honk300.app"
STATE_ROOT="${HOME}/Library/Application Support/honk300"
TRANSITION_DEADLINE_MS=1500
MANAGED_APP_CONFIRMED=0
FINALIZED_MANAGED_FIXTURE=0
SIGNED_APP_DIGEST=""
PROMPT_ROOT=""
PROMPT_MARKER=""
PROMPT_MARKER_STAMP=""

cleanup() {
  result=$?
  trap - EXIT INT TERM
  if [ -x "${BIN}" ]; then
    "${BIN}" stop >/dev/null 2>&1 || true
  fi
  if [ "${MANAGED_APP_CONFIRMED}" = "1" ] \
    && [ "${FINALIZED_MANAGED_FIXTURE}" != "1" ]; then
    if ! finalize_managed_fixture; then
      echo "smoke_m16_macos_accessibility: managed fixture cleanup failed" >&2
      result=1
    fi
  fi
  rm -rf "${TEMP_ROOT}"
  exit "${result}"
}
trap cleanup EXIT INT TERM

die() {
  echo "smoke_m16_macos_accessibility: $*" >&2
  exit 1
}

start_runtime() {
  /usr/bin/open -n "${APP}" --args start --config "${CONFIG}"

  ready=0
  attempts=0
  while [ "${attempts}" -lt 80 ]; do
    if "${BIN}" status >"${STATUS}" 2>&1 && grep -q "honk300: running" "${STATUS}"; then
      ready=1
      break
    fi
    attempts=$((attempts + 1))
    sleep 0.25
  done
  if [ "${ready}" -ne 1 ]; then
    cat "${STATUS}" >&2 || true
    die "runtime did not answer status"
  fi
}

stop_runtime() {
  "${BIN}" stop >/dev/null 2>&1 || true
}

require_operator_evidence() {
  token="$1"
  instruction="$2"
  if [ ! -r /dev/tty ] || [ ! -w /dev/tty ]; then
    die "${token} confirmation requires an interactive terminal: ${instruction}"
  fi
  printf '%s\nType %s and press Return: ' "${instruction}" "${token}" >/dev/tty
  response=""
  if ! IFS= read -r response </dev/tty; then
    die "could not read ${token} confirmation from the terminal"
  fi
  if [ "${response}" != "${token}" ]; then
    die "expected ${token} confirmation, got ${response:-<empty>}"
  fi
  printf 'smoke_m16_macos_accessibility: recorded operator evidence %s\n' "${token}"
}

exercise_single_action() {
  action="$1"
  start_runtime
  cat "${STATUS}"
  grep -q "platform: macOS" "${STATUS}"
  grep -q "accessibility: supported" "${STATUS}"
  grep -q "cursor: supported" "${STATUS}"
  "${BIN}" do "${action}"
  stop_runtime
}

assert_denied_wait() {
  "${BIN}" status >"${STATUS}" 2>&1
  cat "${STATUS}"
  grep -q "platform: macOS" "${STATUS}"
  grep -q "accessibility: denied" "${STATUS}"
  grep -q "cursor: denied" "${STATUS}"
  grep -q "window: denied" "${STATUS}"
  "${BIN}" reload
  "${BIN}" do honk
  for action in wander mud nab meme note; do
    if busy_output="$("${BIN}" do "${action}" 2>&1)"; then
      die "denied permission wait accepted ${action}"
    fi
    printf '%s\n' "${busy_output}" | grep -q "BUSY"
  done
}

assert_prompt_marker() {
  if [ ! -f "${PROMPT_MARKER}" ] || [ -L "${PROMPT_MARKER}" ]; then
    die "prompt marker missing or unsafe: ${PROMPT_MARKER}"
  fi
  if [ "$(stat -f '%Lp' "${PROMPT_MARKER}")" != "600" ]; then
    die "prompt marker is not mode 600"
  fi
  if [ "$(stat -f '%u' "${PROMPT_MARKER}")" != "$(id -u)" ]; then
    die "prompt marker is not owned by the current user"
  fi
  for directory in "${STATE_ROOT}/state" "${PROMPT_ROOT}"; do
    if [ -L "${directory}" ] || [ "$(stat -f '%Lp' "${directory}")" != "700" ]; then
      die "prompt state directory is unsafe: ${directory}"
    fi
  done
  if ! grep -qx "${VERSION}" "${PROMPT_MARKER}"; then
    die "prompt marker content does not match version ${VERSION}"
  fi
}

prompt_marker_fingerprint() {
  marker_stat="$(stat -f '%d:%i:%u:%g:%Lp:%m:%B:%z' "${PROMPT_MARKER}")"
  marker_digest="$(shasum -a 256 "${PROMPT_MARKER}" | awk '{print $1}')"
  printf '%s:%s\n' "${marker_stat}" "${marker_digest}"
}

assert_prompt_marker_unchanged() {
  assert_prompt_marker
  current_stamp="$(prompt_marker_fingerprint)"
  if [ "${current_stamp}" != "${PROMPT_MARKER_STAMP}" ]; then
    die "the denied relaunch or live transition rewrote the prompt marker"
  fi
}

assert_same_signed_app() {
  if ! codesign --verify --strict "${APP}"; then
    echo "smoke_m16_macos_accessibility: managed app signature no longer verifies" >&2
    return 1
  fi
  current_digest="$(shasum -a 256 "${BIN}" | awk '{print $1}')"
  if [ -z "${SIGNED_APP_DIGEST}" ] || [ "${current_digest}" != "${SIGNED_APP_DIGEST}" ]; then
    echo "smoke_m16_macos_accessibility: same signed app binary changed during live smoke" >&2
    return 1
  fi
}

monotonic_millis() {
  python3 -c 'import time; print(time.monotonic_ns() // 1_000_000)'
}

wait_for_accessibility_state() {
  expected="$1"
  deadline_ms=$(( $(monotonic_millis) + TRANSITION_DEADLINE_MS ))
  while [ "$(monotonic_millis)" -le "${deadline_ms}" ]; do
    if "${BIN}" status >"${STATUS}" 2>&1; then
      case "${expected}" in
        supported)
          if grep -q "accessibility: supported" "${STATUS}" \
            && grep -q "cursor: supported" "${STATUS}" \
            && grep -q "window: supported" "${STATUS}"; then
            return 0
          fi
          ;;
        denied)
          if grep -q "accessibility: denied" "${STATUS}" \
            && grep -q "cursor: denied" "${STATUS}" \
            && grep -q "window: denied" "${STATUS}"; then
            return 0
          fi
          ;;
        *)
          die "unsupported wait state ${expected}"
          ;;
      esac
    fi
    sleep 0.1
  done
  return 1
}

wait_for_live_grant() {
  echo "smoke_m16_macos_accessibility: grant Accessibility to the same signed app now"
  require_operator_evidence "GRANTED" "Enable Honk300 in Accessibility, then immediately return here."
  transition_started_ms="$(monotonic_millis)"
  if ! wait_for_accessibility_state "supported"; then
    cat "${STATUS}" >&2 || true
    die "live grant was not observed in the same process within ${TRANSITION_DEADLINE_MS} ms of operator acknowledgement"
  fi
  transition_elapsed_ms=$(( $(monotonic_millis) - transition_started_ms ))
  if [ "${transition_elapsed_ms}" -gt "${TRANSITION_DEADLINE_MS}" ]; then
    die "grant transition exceeded the ${TRANSITION_DEADLINE_MS} ms evidence deadline"
  fi
  echo "smoke_m16_macos_accessibility: grant transition observed after ${transition_elapsed_ms} ms (deadline ${TRANSITION_DEADLINE_MS} ms)"
}

wait_for_live_revocation() {
  echo "smoke_m16_macos_accessibility: revoke Accessibility from the still-running same signed app now"
  require_operator_evidence "REVOKED" "Disable Honk300 in Accessibility, then immediately return here."
  transition_started_ms="$(monotonic_millis)"
  if ! wait_for_accessibility_state "denied"; then
    cat "${STATUS}" >&2 || true
    die "live revocation was not observed in the same process within ${TRANSITION_DEADLINE_MS} ms of operator acknowledgement"
  fi
  transition_elapsed_ms=$(( $(monotonic_millis) - transition_started_ms ))
  if [ "${transition_elapsed_ms}" -gt "${TRANSITION_DEADLINE_MS}" ]; then
    die "revocation transition exceeded the ${TRANSITION_DEADLINE_MS} ms evidence deadline"
  fi
  echo "smoke_m16_macos_accessibility: revocation transition observed after ${transition_elapsed_ms} ms (deadline ${TRANSITION_DEADLINE_MS} ms)"
}

remove_prompt_marker_safely() {
  if [ "${PROMPT_MARKER}" != "${PROMPT_ROOT}/${VERSION}" ]; then
    die "refusing prompt-marker removal outside ${PROMPT_ROOT}"
  fi
  for directory in "${STATE_ROOT}" "${STATE_ROOT}/state" "${PROMPT_ROOT}"; do
    if [ -L "${directory}" ]; then
      die "refusing prompt-marker removal through symlinked directory ${directory}"
    fi
  done
  if [ -L "${PROMPT_MARKER}" ] || { [ -e "${PROMPT_MARKER}" ] && [ ! -f "${PROMPT_MARKER}" ]; }; then
    die "refusing unsafe prompt-marker removal at ${PROMPT_MARKER}"
  fi
  rm -f "${PROMPT_MARKER}"
}

prepare_first_prompt_fixture() {
  if [ "${HONK300_SKIP_BUILD:-0}" != "1" ]; then
    die "live smoke requires HONK300_SKIP_BUILD=1 so the installed signed app is never rebuilt"
  fi
  if [ "${HONK300_RESET_PROMPT_MARKER:-0}" != "1" ]; then
    die "live smoke requires explicit HONK300_RESET_PROMPT_MARKER=1"
  fi
  case "${HONK300_FINAL_CLEANUP:-keep}" in
    keep|purge-managed-install) ;;
    *) die "HONK300_FINAL_CLEANUP must be keep or purge-managed-install" ;;
  esac

  stop_runtime
  SIGNED_APP_DIGEST="$(shasum -a 256 "${BIN}" | awk '{print $1}')"
  assert_same_signed_app
  case "${HONK300_RESET_TCC:-0}" in
    1)
      /usr/bin/tccutil reset Accessibility "${BUNDLE_ID}"
      ;;
    0)
      require_operator_evidence "DENIED_READY" "Disable Honk300 in Accessibility before the first launch."
      ;;
    *)
      die "HONK300_RESET_TCC must be 0 or 1"
      ;;
  esac
  remove_prompt_marker_safely
  if [ -e "${PROMPT_MARKER}" ] || [ -L "${PROMPT_MARKER}" ]; then
    die "prompt marker still exists after scoped reset"
  fi
}

finalize_managed_fixture() {
  case "${HONK300_FINAL_CLEANUP:-keep}" in
    keep)
      echo "smoke_m16_macos_accessibility: managed app and Accessibility state retained (set HONK300_FINAL_CLEANUP=purge-managed-install for destructive cleanup)"
      FINALIZED_MANAGED_FIXTURE=1
      ;;
    purge-managed-install)
      if [ "${MANAGED_APP_CONFIRMED}" != "1" ]; then
        echo "smoke_m16_macos_accessibility: refusing cleanup because the exact managed app was not confirmed" >&2
        return 1
      fi
      if ! assert_same_signed_app; then
        return 1
      fi
      stop_runtime
      "${BIN}" uninstall --purge
      if ! /usr/bin/tccutil reset Accessibility "${BUNDLE_ID}"; then
        echo "smoke_m16_macos_accessibility: tccutil could not reset Honk300 Accessibility during final cleanup" >&2
        return 1
      fi
      if [ -e "${EXPECTED_APP}" ] || [ -L "${EXPECTED_APP}" ]; then
        echo "smoke_m16_macos_accessibility: managed app remains after purge" >&2
        return 1
      fi
      if [ -n "${PROMPT_MARKER}" ] && { [ -e "${PROMPT_MARKER}" ] || [ -L "${PROMPT_MARKER}" ]; }; then
        echo "smoke_m16_macos_accessibility: prompt marker remains after purge" >&2
        return 1
      fi
      FINALIZED_MANAGED_FIXTURE=1
      echo "smoke_m16_macos_accessibility: purged the managed app/state and reset only ${BUNDLE_ID} Accessibility"
      ;;
    *)
      echo "smoke_m16_macos_accessibility: invalid cleanup mode" >&2
      return 1
      ;;
  esac
}

run_live_smoke() {
  prepare_first_prompt_fixture

  start_runtime
  assert_denied_wait
  assert_prompt_marker
  PROMPT_MARKER_STAMP="$(prompt_marker_fingerprint)"
  assert_same_signed_app
  require_operator_evidence "PROMPTED" "Confirm the native Accessibility request and Accessibility Settings appeared once, with the goose waiting at the safe edge."
  stop_runtime
  assert_same_signed_app

  start_runtime
  assert_denied_wait
  assert_prompt_marker_unchanged
  assert_same_signed_app
  sleep 2
  require_operator_evidence "NON_NAG" "Confirm the second denied launch did not reopen the native request or Settings."

  wait_for_live_grant
  assert_same_signed_app
  cat "${STATUS}"
  require_operator_evidence "FIRSTUX" "Confirm the same running goose left the safe edge and resumed its FirstUX introduction."

  wait_for_live_revocation
  assert_denied_wait
  assert_prompt_marker_unchanged
  assert_same_signed_app
  sleep 2
  require_operator_evidence "REVOCATION_QUIET" "Confirm revocation returned the goose to its safe wait without reopening native UI."
  stop_runtime

  finalize_managed_fixture
  echo "smoke_m16_macos_accessibility: prompt, non-nag, same-process grant, and live-revocation smoke passed"
}

case "${PHASE}" in
  denied|live)
    if [ "${HONK300_SKIP_BUILD:-0}" != "1" ]; then
      die "${PHASE} phase requires HONK300_SKIP_BUILD=1 and the already-installed exact signed app"
    fi
    ;;
  granted) ;;
  *)
    die "HONK300_ACCESSIBILITY_PHASE must be denied, live, or granted"
    ;;
esac

if [ "${HONK300_SKIP_BUILD:-0}" = "1" ]; then
  echo "smoke_m16_macos_accessibility: using exact prebuilt app ${APP}"
else
  echo "smoke_m16_macos_accessibility: building universal2 app"
  bash "${ROOT}/script/package_macos_app.sh"
fi

if [ ! -d "${APP}" ] || [ ! -x "${BIN}" ]; then
  echo "smoke_m16_macos_accessibility: app bundle or executable is missing: ${APP}" >&2
  exit 1
fi

codesign --verify --strict "${APP}"
lipo "${BIN}" -verify_arch x86_64 arm64

VERSION="$(/usr/libexec/PlistBuddy -c 'Print :CFBundleShortVersionString' "${APP}/Contents/Info.plist")"
if ! printf '%s\n' "${VERSION}" | grep -Eq '^[0-9]+\.[0-9]+\.[0-9]+$'; then
  die "unsafe app version for prompt state: ${VERSION}"
fi
APP_BUNDLE_ID="$(/usr/libexec/PlistBuddy -c 'Print :CFBundleIdentifier' "${APP}/Contents/Info.plist")"
if [ "${APP_BUNDLE_ID}" != "${BUNDLE_ID}" ]; then
  die "unexpected app bundle identifier ${APP_BUNDLE_ID}"
fi
PROMPT_ROOT="${STATE_ROOT}/state/accessibility-prompt-v1"
PROMPT_MARKER="${PROMPT_ROOT}/${VERSION}"

case "${PHASE}" in
  denied|live)
    if [ ! -d "${EXPECTED_APP}" ] || [ -L "${EXPECTED_APP}" ]; then
      die "${PHASE} phase requires the real managed app ${EXPECTED_APP}"
    fi
    if [ "$(cd "${APP}" && pwd -P)" != "$(cd "${EXPECTED_APP}" && pwd -P)" ]; then
      die "${PHASE} phase requires the exact managed app ${EXPECTED_APP}"
    fi
    SIGNING_INFO="$(codesign -dv --verbose=4 "${APP}" 2>&1)"
    TEAM_ID="$(printf '%s\n' "${SIGNING_INFO}" | sed -n 's/^TeamIdentifier=//p' | head -1)"
    if [ -z "${TEAM_ID}" ] || [ "${TEAM_ID}" = "not set" ]; then
      die "${PHASE} phase requires a Developer ID-signed managed app"
    fi
    SIGNED_APP_DIGEST="$(shasum -a 256 "${BIN}" | awk '{print $1}')"
    MANAGED_APP_CONFIRMED=1
    ;;
  granted) ;;
esac

"${BIN}" setup --config "${CONFIG}"

case "${PHASE}" in
  denied)
    start_runtime
    assert_denied_wait
    assert_prompt_marker
    assert_same_signed_app
    stop_runtime
    echo "smoke_m16_macos_accessibility: denied permission-wait smoke passed"
    ;;
  live)
    run_live_smoke
    ;;
  granted)
    start_runtime
    cat "${STATUS}"
    grep -q "platform: macOS" "${STATUS}"
    grep -q "accessibility: supported" "${STATUS}"
    grep -q "cursor: supported" "${STATUS}"

    "${BIN}" do honk
    "${BIN}" do mud
    "${BIN}" reload
    stop_runtime

    exercise_single_action nab
    exercise_single_action meme
    exercise_single_action note

    echo "smoke_m16_macos_accessibility: Accessibility-granted command smoke passed"
    ;;
esac
