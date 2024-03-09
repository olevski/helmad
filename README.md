# Helmad

An IDE for Helm built with Rust Tauri and HTMX.

This is a work in progress, so feel free to try this out but keep a few things in mind:
- features are still missing (especially when it comes to error handling)
- the styling can be a bit rough in places

Potential future features:
- error handling and surfacing of error messages
- the ability to edit files for local Helm charts right in the app
- watching and automatic refresh of templated resources when a file updates
- allow downloading and editing remote charts
- download and use different Helm CLI versions
- publish installers, executables for different operating systems from CI pipelines

# Instructions

You have to have Rust (including Cargo) and Node installed.

In addition the app will not work if the Helm CLI is not present in your PATH.

1. `npm install`
2. `cargo tauri dev`

