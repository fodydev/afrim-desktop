# Maintainer: Brady Fomegne <brady.fomegne@outlook.com>

pkgname=fcitx5-afrim-desktop
pkgver=0.1.0
pkgrel=1
pkgdesc='Fcitx5 addon for afrim-desktop, a cross-platform input method powered by afrim'
arch=('x86_64')
url='https://github.com/fodydev/afrim-desktop'
license=('MIT' 'Apache-2.0')
depends=('fcitx5' 'libxkbcommon')
makedepends=(
  'rust'
  'cargo'
  'cmake'
  'extra-cmake-modules'
  'pkg-config'
  'gcc'
)

build() {
  cd "$startdir/afrim-desktop-im/fcitx5-addon"
  # The CMake project builds the Rust cdylib itself
  cmake -B build -DCMAKE_INSTALL_PREFIX=/usr
  cmake --build build -j
}

package() {
  cd "$startdir/afrim-desktop-im/fcitx5-addon"
  DESTDIR="$pkgdir" cmake --install build
}
