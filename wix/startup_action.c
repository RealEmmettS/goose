/* Select only the startup feature from the current owned mechanism, after MSI's
 * historical feature migration and before the user chooses features. This DLL
 * is embedded in the MSI, never installed, and never writes registry/config.
 * Windows Installer retains responsibility for transactional Run-value writes.
 */
#define UNICODE
#define _UNICODE
#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#include <msi.h>
#include <msiquery.h>
#include <strsafe.h>
#include <wchar.h>

#define TEXT_CAPACITY 32768

static UINT log_failure(MSIHANDLE install, const wchar_t *message)
{
    MSIHANDLE record = MsiCreateRecord(1);
    if (record != 0) {
        MsiRecordSetStringW(record, 0, L"Goose startup selection: [1]");
        MsiRecordSetStringW(record, 1, message);
        MsiProcessMessage(install, INSTALLMESSAGE_ERROR, record);
        MsiCloseHandle(record);
    }
    return ERROR_INSTALL_FAILURE;
}

static UINT property(MSIHANDLE install, const wchar_t *name, wchar_t *value)
{
    DWORD length = TEXT_CAPACITY;
    return MsiGetPropertyW(install, name, value, &length);
}

static BOOL names_startup(const wchar_t *features)
{
    const wchar_t *item = features;
    while (*item != L'\0') {
        const wchar_t *end = wcschr(item, L',');
        size_t length = end == NULL ? wcslen(item) : (size_t)(end - item);
        /* MSI feature names are case-sensitive; do not reinterpret other names. */
        if ((length == ARRAYSIZE(L"Autostart") - 1 && wcsncmp(item, L"Autostart", length) == 0) ||
            (length == 3 && wcsncmp(item, L"ALL", length) == 0)) {
            return TRUE;
        }
        if (end == NULL) break;
        item = end + 1;
    }
    return FALSE;
}

static UINT explicit_startup(MSIHANDLE install, BOOL *explicit_choice)
{
    static const wchar_t *properties[] = {
        L"ADDLOCAL", L"REMOVE", L"ADDSOURCE", L"ADDDEFAULT", L"ADVERTISE"
    };
    wchar_t value[TEXT_CAPACITY];
    size_t index;
    *explicit_choice = FALSE;
    for (index = 0; index < ARRAYSIZE(properties); ++index) {
        UINT result = property(install, properties[index], value);
        if (result != ERROR_SUCCESS) return result;
        if (names_startup(value)) *explicit_choice = TRUE;
    }
    /* REINSTALL=ALL is a repair, not a renewed opt-in. Its remembered startup
     * feature must be corrected from the current mechanism like an upgrade. */
    return ERROR_SUCCESS;
}

__declspec(dllexport) UINT __stdcall PrepareGooseStartup(MSIHANDLE install)
{
    wchar_t root[TEXT_CAPACITY];
    wchar_t origin[TEXT_CAPACITY];
    wchar_t actual[TEXT_CAPACITY];
    wchar_t expected[TEXT_CAPACITY];
    wchar_t legacy[TEXT_CAPACITY];
    HKEY hive;
    HKEY run = NULL;
    DWORD type = 0;
    DWORD size = sizeof(actual);
    LSTATUS status;
    BOOL enabled = FALSE;
    BOOL explicit_choice = FALSE;
    BOOL prepared;
    BOOL repairing;
    size_t root_length;

    if (property(install, L"APPLICATIONFOLDER", root) != ERROR_SUCCESS ||
        property(install, L"Honk300StartupScope", origin) != ERROR_SUCCESS) {
        return log_failure(install, L"Cannot read the bounded installer identity.");
    }
    /* A private, fixed package property binds this check to the registry
     * component's hive. Public HONK300ORIGIN is caller-overridable. */
    if (wcscmp(origin, L"machine") == 0) {
        hive = HKEY_LOCAL_MACHINE;
    } else if (wcscmp(origin, L"user") == 0) {
        hive = HKEY_CURRENT_USER;
    } else {
        return log_failure(install, L"Unknown installer ownership.");
    }
    root_length = wcslen(root);
    while (root_length > 0 && (root[root_length - 1] == L'\\' || root[root_length - 1] == L'/')) {
        root[--root_length] = L'\0';
    }
    if (root_length < 3 || root[1] != L':' || root[2] != L'\\' ||
        FAILED(StringCchPrintfW(expected, TEXT_CAPACITY, L"\"%s\\bin\\honk300-app.exe\"", root)) ||
        FAILED(StringCchPrintfW(legacy, TEXT_CAPACITY, L"\"%s\\bin\\honk300.exe\" start", root))) {
        return log_failure(install, L"The resolved application directory is invalid or too long.");
    }

    status = RegOpenKeyExW(hive, L"Software\\Microsoft\\Windows\\CurrentVersion\\Run",
                          0, KEY_QUERY_VALUE | KEY_WOW64_64KEY, &run);
    if (status == ERROR_SUCCESS) {
        status = RegQueryValueExW(run, L"Honk300", NULL, &type, (BYTE *)actual, &size);
        RegCloseKey(run);
        if (status == ERROR_SUCCESS) {
            if (type != REG_SZ || size < sizeof(wchar_t) || size > sizeof(actual) ||
                size % sizeof(wchar_t) != 0 || actual[size / sizeof(wchar_t) - 1] != L'\0' ||
                wcslen(actual) + 1 != size / sizeof(wchar_t)) {
                return log_failure(install, L"The existing startup value has an unsupported type or encoding.");
            }
            if (_wcsicmp(actual, expected) != 0 && _wcsicmp(actual, legacy) != 0) {
                return log_failure(install, L"The existing startup value belongs to another command; installation did not change it.");
            }
            enabled = TRUE;
        }
    }
    if (status != ERROR_SUCCESS && status != ERROR_FILE_NOT_FOUND) {
        return log_failure(install, L"Cannot safely read the existing startup value.");
    }
    if (explicit_startup(install, &explicit_choice) != ERROR_SUCCESS) {
        return log_failure(install, L"Cannot read explicit feature selections.");
    }
    if (property(install, L"HONK300STARTUPPREPARED", origin) != ERROR_SUCCESS) {
        return log_failure(install, L"Cannot read the startup selection phase.");
    }
    prepared = origin[0] != L'\0';
    if (property(install, L"REINSTALL", origin) != ERROR_SUCCESS) {
        return log_failure(install, L"Cannot read the repair selection.");
    }
    repairing = origin[0] != L'\0';
    if ((!prepared || repairing) && !explicit_choice && MsiSetFeatureStateW(install, L"Autostart",
            enabled ? INSTALLSTATE_LOCAL : INSTALLSTATE_ABSENT) != ERROR_SUCCESS) {
        return log_failure(install, L"Cannot apply the current startup choice to this installer.");
    }
    /* Secure property carries UI initialization into execute. Otherwise execute
     * would overwrite the user's later interactive feature-tree selection. */
    if (MsiSetPropertyW(install, L"HONK300STARTUPPREPARED", L"1") != ERROR_SUCCESS) {
        return log_failure(install, L"Cannot retain the initialized startup selection.");
    }
    return ERROR_SUCCESS;
}
