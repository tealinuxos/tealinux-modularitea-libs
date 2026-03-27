pkgname=modularitea-libs
pkgver=1.0
pkgrel=1
pkgdesc="Modularitea libraries & various helper func"
arch=('x86_64')
license=('GPL')
depends=()
source=()
sha256sums=()

build() {
    set -a
    source "$startdir/build.env"
    set +a
    
    # export AWS_LC_RS_NO_PREFIX=
    # export AWS_LC_SYS_NO_ASM=1
    export CARGO_TARGET_DIR="$srcdir/target"
    cargo build --release -vv
}

package() {
    install -Dm755 "$srcdir/target/release/modularitea-profile-installer" \
    "$pkgdir/usr/bin/modularitea-profile-installer"

    install -Dm755 "$srcdir/target/release/modularitea-grub" \
        "$pkgdir/usr/bin/modularitea-grub"

    install -d "$pkgdir/usr/share/modularitea-libs"
    cp -a "$srcdir/data/." "$pkgdir/usr/share/modularitea-libs/"
}