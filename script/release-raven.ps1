$version = Select-String -Path .\Cargo.toml -Pattern "^version = ""([^""]+)""$"
$version = $version.Matches.Groups[1].Value

echo $version

cargo +stable build --release
mv target/release/comrak.exe comrak-$version-x86_64-pc-windows-msvc.exe

# rustup +stable toolchain install stable-x86_64-pc-windows-gnu
# $env:Path += ";C:\msys64\usr\bin"
# rustup run stable-gnu cargo build --release --target=x86_64-pc-windows-gnu
# mv target/x86_64-pc-windows-gnu/release/comrak.exe comrak-$version-x86_64-pc-windows-gnu.exe
