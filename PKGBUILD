pkgname=musicat
pkgver=0.12.0
pkgrel=1
pkgdesc="A sleek desktop music player and tagger for offline music 🪕. With gapless playback, smart playlists, and a map view! Built with Svelte and Tauri "
arch=('x86_64')

depends=('gtk3' 'webkit2gtk' 'libayatana-appindicator')
makedepends=('rpmextract')
url="https://github.com/basharovV/musicat"
license=('GPL3')

source=("$pkgname-$pkgver.rpm::https://github.com/basharovV/musicat/releases/download/v${pkgver}/Musicat-${pkgver}-1.x86_64.rpm")
sha256sums=('SKIP')  # Replace SKIP with actual checksum after downloading

package() {
    cd "$srcdir"
    rpmextract.sh "$pkgname-$pkgver.rpm"
    cp -r usr/ "$pkgdir/"
}