# Maintainer: Dennis Mellert <dennis.mellert@gmail.com>
_pkgname=pkghist
pkgname=pkghist-git
pkgver=r34.4bc5ccb
pkgrel=1
pkgdesc="Query the version(s) of installed and removed packages"
source=("git+https://github.com/herzrasen/${_pkgname}")
makedepends=('git' 'rust')
arch=('i686' 'x86_64' 'armv6h' 'armv7h')
sha512sums=('SKIP')

pkgver() {
  cd "${srcdir}/${_pkgname}"
  printf "r%s.%s" "$(git rev-list --count HEAD)" "$(git rev-parse --short HEAD)"
}

build() {
  cd "${srcdir}/${_pkgname}"
  cargo build --release
}

package() {
  cd "${srcdir}/${_pkgname}"
  install -D target/release/${_pkgname} "${pkgdir}/usr/bin/${_pkgname}"
  install -Dm 644 LICENSE -t "${pkgdir}/usr/share/licenses/pkghist"
}
