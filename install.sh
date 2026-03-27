#!/usr/bin/bash
srcdir=.

export CARGO_TARGET_DIR="$srcdir/target"
cargo build --release -vv


install -Dm755 "$srcdir/target/release/modularitea-profile-installer" \
"$pkgdir/usr/bin/modularitea-profile-installer"

install -Dm755 "$srcdir/target/release/modularitea-grub" \
    "$pkgdir/usr/bin/modularitea-grub"

install -d "$pkgdir/usr/share/modularitea-libs"
cp -a "$srcdir/data/." "$pkgdir/usr/share/modularitea-libs/"
