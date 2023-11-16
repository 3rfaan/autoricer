# Maintainer: Arfan Zubi
# Maintainer: <zubi.arfan@gmail.com
#
# This PKGBUILD was generated by `cargo aur`: https://crates.io/crates/cargo-aur

pkgname=hyprforest-installer-bin
pkgver=0.1.0
pkgrel=1
pkgdesc="Everforest Theme Installer for Hyprland on Arch Linux"
url="https://github.com/3rfaan/hyprforest-installer"
license=("GPL-3.0-or-later")
arch=("x86_64")
provides=("hyprforest-installer")
conflicts=("hyprforest-installer")
optdepends=("hyprland" "kitty" "neovim" "waybar" "wofi" "zsh")
source=("https://github.com/3rfaan/hyprforest-installer/releases/download/v$pkgver/hyprforest-installer-$pkgver-x86_64.tar.gz")
sha256sums=("591c246ec182e8fd048ed0ba996f4e1da159a78528ecd998cb6ba672b34ed70d")

package() {
    install -Dm755 hyprforest-installer -t "$pkgdir/usr/bin"
}
