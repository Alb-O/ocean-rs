{ pkgs, ... }:
let
  # Bevy runtime dependencies
  bevyDeps = with pkgs; [
    # Graphics
    vulkan-loader
    vulkan-headers
    vulkan-tools
    vulkan-validation-layers

    # Audio
    alsa-lib

    # Windowing
    libxkbcommon
    wayland

    # X11
    xorg.libX11
    xorg.libXcursor
    xorg.libXi
    xorg.libXrandr

    # Input (gamepad support)
    udev
  ];
in
{
  bevy = pkgs.mkShell {
    packages = [ pkgs.pkg-config ] ++ bevyDeps;

    # Enable dynamic linking for faster compile times
    RUSTFLAGS = "-C linker=clang -C link-arg=-fuse-ld=mold";

    # shellHook is inherited via inputsFrom
    shellHook = ''
      export LD_LIBRARY_PATH="${pkgs.lib.makeLibraryPath bevyDeps}:/run/opengl-driver/lib''${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
    '';
  };
}
