set windows-shell := ["pwsh", "-NoLogo", "-NoProfileLoadTime", "-Command"]
lib_name := "napi_sandbox"

profile := env_var_or_default("profile", "debug")

profile_cargo := \
if \
  profile != "debug" { "--profile " + profile } \
else \
  { "" }


dylib := \
if \
  os() == "windows" { lib_name + ".dll" } \
else if \
  os() == "macos" { "lib" + lib_name + ".dylib" } \
else if \
  os() == "linux" { "lib" + lib_name + ".so" } \
else \
  { os() }

root_dir :=  justfile_directory()
out_dir :=  join(justfile_directory(), "target", profile)

build:
    @test -d node_modules || npm install
    cargo build {{profile_cargo}}
    rm -rf {{root_dir}}/examples/addon/index.node
    mv {{out_dir}}/{{dylib}} {{root_dir}}/examples/addon/index.node

run example="example-a":
    just build
    node {{root_dir}}/examples/nodejs/{{example}}.js

fmt:
  cargo +nightly fmt
  
publish:
  cargo publish -p napi_ext_macros
  cargo publish -p napi_ext