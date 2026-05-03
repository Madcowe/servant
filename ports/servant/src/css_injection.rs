/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

pub const INJECTED_CSS: &str = r#"
<style>
    @keyframes ant-shimmer {
        0% { background-position: 150% 0; }
        100% { background-position: -150% 0; }
    }
    img[src^="ant://"] { 
        background: linear-gradient(90deg, #f0f0f0 25%, #e8e8e8 50%, #f0f0f0 75%);
        background-size: 200% 100%;
        animation: ant-shimmer 2s infinite linear;
        min-width: 32px;
        min-height: 32px;
        border-radius: 4px;
        display: inline-block;
    }
</style>
"#;
