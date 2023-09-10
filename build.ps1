cargo build -r

if (Test-Path -Path "dist") {
  Remove-Item -Path "dist" -Recurse -Force
}

New-Item -Path "dist" -ItemType "directory"

Copy-Item -Path "target/release/aivoice-vox.exe" -Destination "dist/aivoice-vox.exe" -Force

Copy-Item -Path ./open_jtalk_dic_utf_8-1.11 ./dist/open_jtalk_dic_utf_8-1.11 -Recurse -Force
Copy-Item -Path ./README.md -Destination ./dist/README.md
Copy-Item -Path ./engine_manifest.json -Destination ./dist/engine_manifest.json

Set-Location -Path "dist"

Compress-Archive -Path * -DestinationPath "../aivoice-vox.zip" -Force
cd ..
Move-Item -Path "./aivoice-vox.zip" -Destination "./aivoice-vox.vvpp" -Force
