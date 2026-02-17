pkgname=modularitea-libs
pkgver=1.0
pkgrel=1
pkgdesc="Modularitea libraries"
arch=('x86_64')
license=('GPL')
depends=()
source=()
sha256sums=()

build() {
    export CARGO_TARGET_DIR=target
    cargo build --release --locked
}

package() {
    install -Dm755 ./target/release/modularitea-profile-installer \
        "$pkgdir/usr/bin/modularitea-profile-installer"
}