TT;DR: The entire Windows installer EULA is saved as Markdown in Downloads and published at https://thegoose.app/eula through the footer. The complete page and public Markdown download match the verified installer export.

## Why
The user wants to keep and share the installer text, including the PolyForm terms and the complete non-binding Great Honk Accord.

## Scope
Preserve every paragraph and notice from the native installer. Use the existing website session to publish a dedicated page with a footer link, without adding a top navigation entry.

## Plan
- Verify the published MSI license control against the canonical RTF.
- Convert formatting to Markdown without changing the wording and save it in Downloads.
- Send the exact source and navigation requirements to the website session.
- Verify the production page and record the result.

## Acceptance
- The export includes the acceptance notice, complete PolyForm license, third-party media notice, all sixteen ceremonial articles and final paragraph.
- Markdown text matches the native installer text after formatting normalization.
- The website displays the complete text through a working footer link and direct page address.

## Evidence
- Canonical installer source: `wix/honk300-license.rtf`.
- Published source release: `v1.11.0`; the RTF is unchanged from `v1.10.1`.
- Revalidated all four published MSI hashes and opened their `LicenseAgreementDlg/LicenseText` controls read-only. Every embedded RTF matches the canonical source after newline normalization.
- Compared every Markdown paragraph with Windows native RichTextBox decoding. All sixteen articles and the final paragraph are present.
- Saved `C:/Users/hey/Downloads/Goose EULA.md`: 19,721 UTF-8 bytes, SHA-256 `ea3f46dec8c72c351a0ae1b43aca31e6b8fcdd703d48ef648d6ee52a0c9e4046`.
- Local extraction and artifact evidence: `target/eula-export/evidence.json`.
- Sent the verified file and footer-only requirement to the existing **Redesign desktop Goose site** session, `01a07ae1-cdc5-7050-979e-2defc48e27a0`.
- Website [PR #15](https://github.com/RealEmmettS/desktop-goose-site/pull/15) merged as `f6e873870d6196dcbe9dfad390778d62b7b60545`; production deployment `6355956313` succeeded. All four final PR jobs in [34385083549](https://github.com/RealEmmettS/desktop-goose-site/actions/runs/34385083549) and all four post-merge jobs in [34385318425](https://github.com/RealEmmettS/desktop-goose-site/actions/runs/34385318425) passed.
- Root independently fetched the public `/eula` article and `/Goose%20EULA.md`. Complete rendered text matches after Markdown/autolink and whitespace normalization; all sixteen articles and the final sentence are present. The public Markdown has the exact export size and SHA-256 above. Evidence: `target/eula-export/production-verification.json`.
- Root followed the live Chrome homepage footer EULA link to `/eula`, checked the unchanged Install/Questions/Source header, and visually inspected the final paragraph and footer. Website qualification additionally covers mobile, enlarged text, native platform screenshots, accessibility, JavaScript-free direct loading, and the existing artwork/download identities.
- Website handoff receipt: `C:/Users/hey/git/desktop-goose-site/output/design/eula-release-handoff.json`.

## Verification
- [x] All four published MSI hashes match the recorded public release and their license controls match the canonical RTF.
- [x] Native RTF decoding and saved Markdown text agree; all sixteen articles and the final paragraph are present.
- [x] The complete Markdown file is saved in Downloads and the website session has the verified source and footer-only requirement.
- [x] The production EULA page displays the complete text, loads directly and works through the footer without a top navigation entry.

## Status
Completed. The Downloads export and public EULA page are delivered and verified. This request does not change application installation or the separate pending Windows acceptance task.

## Activity
- 2026-09-09 — Website PR #15 publishes the complete EULA after native screenshot review and final checks. Independently verify live footer navigation, full article text, final sentence and identical downloadable bytes; confirm all post-merge website checks pass and close the task.
- 2026-09-09 — Save and read back the full Markdown export after native decoding and four published-MSI comparisons. Ask the existing website session to publish the exact complete text at a dedicated footer-accessible page and verify production behavior.
- 2026-09-09 — User requests the complete installer EULA in Downloads and asks the existing website session to add a footer-accessible page. Confirm both MSI editions use the same canonical RTF and the text is unchanged between the last two releases.
