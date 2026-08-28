# SPDX-FileCopyrightText: 2026 The Anytag Backend Authors
# SPDX-License-Identifier: AGPL-3.0-only

diesel +args:
	mise x -- diesel {{ args }}

dev *args:
    mise x -- cargo run {{ args }}

test *args:
    mise x -- cargo test {{ args }}

watch:
    mise x -- cargo watch -x run

cargo *args:
    mise x -- cargo {{ args }}

reuse *args:
    mise x -- reuse {{ args }}
