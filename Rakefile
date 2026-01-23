# frozen_string_literal: true
require "bundler/setup"

task :build, [:version] do |t, args|
  require "json"
  require "zip"

  if args[:version].nil?
    puts "Usage: rake build[version]"
    exit 1
  end

  sh "cargo build -r"
  rm_rf "dist"
  mkdir "dist"

  cp "./target/release/aivoice-vox.exe", "./dist/aivoice-vox.exe"
  cp_r "./open_jtalk_dic_utf_8-1.11", "./dist/open_jtalk_dic_utf_8-1.11"
  cp "./README.md", "./dist/README.md"

  engine_manifest = JSON.parse(File.read("./engine_manifest.json"))
  File.write("./dist/engine_manifest.json", { **engine_manifest, version: args[:version] }.to_json)

  rm_f "aivoice-vox-#{args[:version]}.vvpp"
  Zip::File.open("aivoice-vox-#{args[:version]}.vvpp", create: true) do |zipfile|
    Dir["./dist/**/**"].each do |file|
      zipfile.add(file.sub("./dist/", ""), file)
    end
  end
  puts "Created aivoice-vox-#{args[:version]}.vvpp"

  base_cargo_toml = File.read("Cargo.toml")
  base_versioned_cargo_toml = base_cargo_toml.sub(/^version = ".*?"$/m, "version = \"#{args[:version]}\"")
  File.write("Cargo.toml", base_versioned_cargo_toml)

  sh "git commit -am \"release: v#{args[:version]}\""
  sh "git tag v#{args[:version]}"
  puts "Updated Cargo.toml to version #{args[:version]}"
end
