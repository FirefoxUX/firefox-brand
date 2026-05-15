# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

{{#if name == official}}
MOZ_APP_DISPLAYNAME=Firefox
{{#elseif name == nightly}}
MOZ_APP_DISPLAYNAME="Firefox Nightly"
MOZ_MACBUNDLE_ID=nightly
{{#elseif name == aurora}}
MOZ_APP_DISPLAYNAME="Firefox Developer Edition"
MOZ_APP_REMOTINGNAME=firefox-dev
MOZ_DEV_EDITION=1
{{#else}}
MOZ_APP_DISPLAYNAME=Nightly
MOZ_MACBUNDLE_ID=nightlyunofficial
{{#endif}}
