# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

{{#if name == official}}
# NSIS branding defines for official release builds.
# The nightly build branding.nsi is located in browser/installer/windows/nsis/
# The unofficial build branding.nsi is located in browser/branding/unofficial/
{{#elseif name == nightly}}
# NSIS branding defines for nightly builds.
# The official release build branding.nsi is located in other-license/branding/firefox/
# The unofficial build branding.nsi is located in browser/branding/unofficial/
{{#elseif name == aurora}}
# NSIS branding defines for Developer Edition builds.
# The official release build branding.nsi is located in other-license/branding/firefox/
# The unofficial build branding.nsi is located in browser/branding/unofficial/
{{#else}}
# NSIS branding defines for unofficial builds.
# The official release build branding.nsi is located in other-license/branding/firefox/
# The nightly build branding.nsi is located in browser/installer/windows/nsis/
{{#endif}}

# BrandFullNameInternal is used for some registry and file system values
# instead of BrandFullName and typically should not be modified.
{{#if name == official}}
!define BrandFullNameInternal "Mozilla Firefox"
{{#elseif name == nightly}}
!define BrandFullNameInternal "Nightly"
{{#elseif name == aurora}}
!define BrandFullNameInternal "Firefox Developer Edition"
{{#else}}
!define BrandFullNameInternal "Mozilla Developer Preview"
{{#endif}}
{{#if name == aurora}}
!define BrandShortName        "Firefox Developer Edition"
{{#endif}}
{{#if name == official}}
!define BrandFullName         "Mozilla Firefox"
{{#elseif name == nightly}}
!define BrandFullName         "Firefox Nightly"
{{#elseif name == aurora}}
!define BrandFullName         "Firefox Developer Edition"
{{#else}}
!define BrandFullName         "Mozilla Developer Preview"
{{#endif}}
{{#if name == official}}
!define CompanyName           "Mozilla Corporation"
{{#else}}
!define CompanyName           "mozilla.org"
{{#endif}}
!define URLInfoAbout          "https://www.mozilla.org"
{{#if name == official}}
!define URLUpdateInfo         "https://www.mozilla.org/firefox/${AppVersion}/releasenotes"
{{#endif}}
!define HelpLink              "https://support.mozilla.org"

{{#if name == official}}
; The OFFICIAL define is a workaround to support different urls for Release and
; Beta since they share the same branding when building with other branches that
; set the update channel to beta.
!define OFFICIAL
{{#endif}}
{{#if name == nightly}}
!define URLStubDownloadX86 "https://download.mozilla.org/?os=win&lang=${AB_CD}&product=firefox-nightly-latest"
!define URLStubDownloadAMD64 "https://download.mozilla.org/?os=win64&lang=${AB_CD}&product=firefox-nightly-latest"
!define URLStubDownloadAArch64 "https://download.mozilla.org/?os=win64-aarch64&lang=${AB_CD}&product=firefox-nightly-latest"
{{#elseif name == aurora}}
!define URLStubDownloadX86 "https://download.mozilla.org/?os=win&lang=${AB_CD}&product=firefox-devedition-latest"
!define URLStubDownloadAMD64 "https://download.mozilla.org/?os=win64&lang=${AB_CD}&product=firefox-devedition-latest"
!define URLStubDownloadAArch64 "https://download.mozilla.org/?os=win64-aarch64&lang=${AB_CD}&product=firefox-devedition-latest"
{{#else}}
!define URLStubDownloadX86 "https://download.mozilla.org/?os=win&lang=${AB_CD}&product=firefox-latest"
!define URLStubDownloadAMD64 "https://download.mozilla.org/?os=win64&lang=${AB_CD}&product=firefox-latest"
!define URLStubDownloadAArch64 "https://download.mozilla.org/?os=win64-aarch64&lang=${AB_CD}&product=firefox-latest"
{{#endif}}
{{#if name == nightly}}
!define URLManualDownload "https://www.mozilla.org/${AB_CD}/firefox/installer-help/?channel=nightly&installer_lang=${AB_CD}"
{{#elseif name == aurora}}
!define URLManualDownload "https://www.mozilla.org/${AB_CD}/firefox/installer-help/?channel=aurora&installer_lang=${AB_CD}"
{{#else}}
!define URLManualDownload "https://www.mozilla.org/${AB_CD}/firefox/installer-help/?channel=release&installer_lang=${AB_CD}"
{{#endif}}
!define URLSystemRequirements "https://www.mozilla.org/firefox/system-requirements/"
{{#if name == official}}
!define Channel "release"
{{#elseif name == nightly}}
!define Channel "nightly"
{{#elseif name == aurora}}
!define Channel "aurora"
{{#else}}
!define Channel "unofficial"
{{#endif}}

# The installer's certificate name and issuer expected by the stub installer
!define CertNameDownload   "Mozilla Corporation"
!define CertIssuerDownload "DigiCert Trusted G4 Code Signing RSA4096 SHA384 2021 CA1"

{{#if name == official}}
# Dialog units are used so the UI displays correctly with the system's DPI
# settings. These are tweaked to look good with the en-US strings; ideally
# we would customize them for each locale but we don't really have a way to
# implement that and it would be a ton of work for the localizers.
!define PROFILE_CLEANUP_LABEL_TOP "50u"
!define PROFILE_CLEANUP_LABEL_LEFT "22u"
!define PROFILE_CLEANUP_LABEL_WIDTH "175u"
!define PROFILE_CLEANUP_LABEL_HEIGHT "100u"
!define PROFILE_CLEANUP_LABEL_ALIGN "left"
!define PROFILE_CLEANUP_CHECKBOX_LEFT "22u"
!define PROFILE_CLEANUP_CHECKBOX_WIDTH "175u"
!define PROFILE_CLEANUP_BUTTON_LEFT "22u"
!define INSTALL_HEADER_TOP "70u"
!define INSTALL_HEADER_LEFT "22u"
!define INSTALL_HEADER_WIDTH "180u"
!define INSTALL_HEADER_HEIGHT "100u"
!define INSTALL_BODY_LEFT "22u"
!define INSTALL_BODY_WIDTH "180u"
!define INSTALL_INSTALLING_TOP "115u"
!define INSTALL_INSTALLING_LEFT "270u"
!define INSTALL_INSTALLING_WIDTH "150u"
!define INSTALL_PROGRESS_BAR_TOP "100u"
!define INSTALL_PROGRESS_BAR_LEFT "270u"
!define INSTALL_PROGRESS_BAR_WIDTH "150u"
!define INSTALL_PROGRESS_BAR_HEIGHT "12u"
{{#else}}
# Dialog units are used so the UI displays correctly with the system's DPI
# settings.
!define PROFILE_CLEANUP_LABEL_TOP "35u"
!define PROFILE_CLEANUP_LABEL_LEFT "0"
!define PROFILE_CLEANUP_LABEL_WIDTH "100%"
!define PROFILE_CLEANUP_LABEL_HEIGHT "80u"
!define PROFILE_CLEANUP_LABEL_ALIGN "center"
!define PROFILE_CLEANUP_CHECKBOX_LEFT "center"
!define PROFILE_CLEANUP_CHECKBOX_WIDTH "100%"
!define PROFILE_CLEANUP_BUTTON_LEFT "center"
!define INSTALL_BLURB_TOP "137u"
!define INSTALL_BLURB_WIDTH "60u"
!define INSTALL_FOOTER_TOP "-48u"
!define INSTALL_FOOTER_WIDTH "250u"
!define INSTALL_INSTALLING_TOP "70u"
!define INSTALL_INSTALLING_LEFT "0"
!define INSTALL_INSTALLING_WIDTH "100%"
!define INSTALL_PROGRESS_BAR_TOP "112u"
!define INSTALL_PROGRESS_BAR_LEFT "20%"
!define INSTALL_PROGRESS_BAR_WIDTH "60%"
!define INSTALL_PROGRESS_BAR_HEIGHT "12u"
{{#endif}}

{{#if name == official}}
!define PROFILE_CLEANUP_CHECKBOX_TOP_MARGIN "12u"
!define PROFILE_CLEANUP_BUTTON_TOP_MARGIN "12u"
!define PROFILE_CLEANUP_BUTTON_X_PADDING "80u"
!define PROFILE_CLEANUP_BUTTON_Y_PADDING "8u"
!define INSTALL_BODY_TOP_MARGIN "20u"
{{#else}}
!define PROFILE_CLEANUP_CHECKBOX_TOP_MARGIN "20u"
!define PROFILE_CLEANUP_BUTTON_TOP_MARGIN "20u"
!define PROFILE_CLEANUP_BUTTON_X_PADDING "40u"
!define PROFILE_CLEANUP_BUTTON_Y_PADDING "4u"
{{#endif}}

# Font settings that can be customized for each channel
{{#if name == official}}
!define INSTALL_HEADER_FONT_SIZE 20
!define INSTALL_HEADER_FONT_WEIGHT 600
!define INSTALL_INSTALLING_FONT_SIZE 15
!define INSTALL_INSTALLING_FONT_WEIGHT 600
{{#else}}
!define INSTALL_HEADER_FONT_SIZE 28
!define INSTALL_HEADER_FONT_WEIGHT 400
!define INSTALL_INSTALLING_FONT_SIZE 28
!define INSTALL_INSTALLING_FONT_WEIGHT 400
{{#endif}}

# UI Colors that can be customized for each channel
{{#if name == official}}
!define COMMON_TEXT_COLOR 0x000000
!define COMMON_BACKGROUND_COLOR 0xFFFFFF
{{#else}}
!define COMMON_TEXT_COLOR 0xFFFFFF
!define COMMON_BACKGROUND_COLOR 0x000000
{{#endif}}
!define INSTALL_INSTALLING_TEXT_COLOR 0xFFFFFF
{{#if name == official}}
# This color is written as 0x00BBGGRR because it's actually a COLORREF value.
!define PROGRESS_BAR_BACKGROUND_COLOR 0xFFAA00
{{#endif}}
{{#if name == aurora}}

# Enable DeveloperEdition-specific behavior
!define DEV_EDITION
{{#endif}}