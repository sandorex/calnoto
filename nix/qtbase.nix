{ stdenvNoCC
, fetchurl
, pkgs
, inputs
, p7zip
, lib
}:

# TODO define androidsdk supported as well
let
  version = "6.10.1";

  # TODO may need to change LD_INTERPRETER etc
  qtbase = stdenvNoCC.mkDerivation {
    pname = "qtbase-android";
    inherit version;

    src = fetchurl {
      url = "https://download.qt.io/online/qtsdkrepository/all_os/android/qt6_6101/qt6_6101_arm64_v8a/qt.qt6.6101.android_arm64_v8a/6.10.1-0-202511161843qtbase-MacOS-MacOS_14-Clang-Android-Android_ANY-ARM64.7z";
      hash = "sha256-EbxYOkqHshHZOorppZtRyUnsMgtCnoC1S2c7SxbHpCM=";
    };

    phases = [ "unpackPhase" ];


    unpackPhase = ''
      ${lib.getExe p7zip} -o$out x $src
    '';

    qtPluginPrefix = "lib/qt-6/plugins";
    qtQmlPrefix = "lib/qt-6/qml";
  };

  qtdeclarative = stdenvNoCC.mkDerivation {
    pname = "qtdeclarative-android";
    inherit version;

    src = fetchurl {
      url = "https://download.qt.io/online/qtsdkrepository/all_os/android/qt6_6101/qt6_6101_arm64_v8a/qt.qt6.6101.android_arm64_v8a/6.10.1-0-202511161843qtdeclarative-MacOS-MacOS_14-Clang-Android-Android_ANY-ARM64.7z";
      hash = "sha256-GcA1+PSaX3NmL2W/Qww+/3wDKLBXCvjPLflG+Fq588I=";
    };

    phases = [ "unpackPhase" ];

    unpackPhase = ''
      ${lib.getExe p7zip} -o$out x $src
    '';
  };

  # https://github.com/NixOS/nixpkgs/blob/ec7c70d12ce2fc37cb92aff673dcdca89d187bae/pkgs/development/libraries/qt-6/qt-env.nix
  buildQtEnv = pkgs.callPackage "${inputs.nixpkgs}/pkgs/development/libraries/qt-6/qt-env.nix" { inherit qtbase; };
in
(buildQtEnv "qt-android-${version}" [
  qtdeclarative
])
