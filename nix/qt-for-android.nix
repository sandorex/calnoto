{ stdenvNoCC
, fetchurl
, fetchzip
, lib
, javaPackages
, androidenv
, cmake
, ninja
, python312
, perl
, qt6
, kdePackages
, libGL
, submodules ? [ "qtdeclarative" ]
}:

# TODO define androidsdk supported as well
let
  version = "6.10.1";
  versionNoPatch =
    let
      parts = lib.splitVersion version;
      major = builtins.elemAt parts 0;
      minor = builtins.elemAt parts 1;
    in
    "${major}.${minor}";

  androidVersion = {
    buildTools = "36.0.0";
    sdk = "36";
    ndk = "27.2.12479018";
    cmake = "4.1.0";
  };

  androidPackages = androidenv.composeAndroidPackages {
    includeNDK = true;
    buildToolsVersions = [ androidVersion.buildTools ];
    platformVersions = [ androidVersion.sdk ];
    ndkVersions = [ androidVersion.ndk ];
    cmakeVersions = [ androidVersion.cmake ];
  };

  # can i get the cmake package like this?
  sdk = androidPackages.androidsdk;

  # android_home = "${sdk}/libexec/android-sdk";
  android_sdk_root = "${sdk}/libexec/android-sdk";
  android_ndk_root = "${sdk}/libexec/android-sdk/ndk/${androidVersion.ndk}";

  android-openssl = fetchzip {
    url = "https://github.com/KDAB/android_openssl/archive/b71f1470962019bd89534a2919f5925f93bc5779.zip";
    hash = "sha256-MnAKh+Iq3wFd0siuHM5sB1ndgUzMO529MQpDKh9k6hE=";
  };

  # https://github.com/NixOS/nixpkgs/blob/ec7c70d12ce2fc37cb92aff673dcdca89d187bae/pkgs/development/libraries/qt-6/qt-env.nix
  # buildQtEnv = pkgs.callPackage "${inputs.nixpkgs}/pkgs/development/libraries/qt-6/qt-env.nix" { inherit qtbase; };
  qtEnv = with qt6; env "qt6-env" [
    qtdeclarative
    kdePackages.qtshadertools
  ];
in
stdenvNoCC.mkDerivation {
  pname = "qt-for-android";
  inherit version;

  src = fetchurl {
    url = "https://download.qt.io/official_releases/qt/${versionNoPatch}/${version}/single/qt-everywhere-src-${version}.tar.xz";
    hash = "sha256-DtCLB5cZOUMDzSBUtmstwMWJXO64j7YTHBiZHJgL8A8=";
  };

  nativeBuildInputs = [
    javaPackages.compiler.openjdk17
    sdk
    cmake
    ninja
    perl
    python312

    libGL
    # androidPackages.cmake
  ];

  # TODO cmake permission denied....
  # TODO idk how to allow access to sdk cmake
    # export PATH="$(echo "$ANDROID_HOME/cmake/${androidVersion.cmake}".*/bin):$PATH"

# prefix is automatic? https://nixos.org/manual/nixpkgs/stable/#sec-stdenv-phases
              # -prefix $out \
  # autoconf is used instead
  dontUseCmakeConfigure = true;
    # "-android-ndk-host" "linux-x86_64"

  configureFlags = [
    "-c++std" "c++17"
    "-opensource"
    "-release"
    "-confirm-license"
    "-xplatform" "android-clang"
    # "-ssl"
    # "-openssl-runtime" 
    # "-I" "${android-openssl}/ssl_3/include"
    # "-L" "${android-openssl}/ssl_3"
    "-qt-host-path" "${qtEnv}"
    "-android-sdk" "${android_sdk_root}"
    "-android-ndk" "${android_ndk_root}"
    "-android-abis" "arm64-v8a"
    "-no-warnings-are-errors"
    "-disable-rpath"
    "-nomake" "tests"
    "-nomake" "examples"
    "-submodules" "${ builtins.concatStringsSep "," submodules }"
    "-I" "${libGL.dev}/include"
    "-L" "${libGL}/lib"
  ];

  # OPENSSL_LIBS="-L${android-openssl}/ssl_3 -I ${android-openssl}/ssl_3/include -lssl";
  # OPENSSL_LIBS='-L/opt/ssl/lib -lssl -lcrypto'

  # configurePhase = ''
  #   mkdir $out
  #   ./configure
  #             -c++std c++17 \
  #             -opensource \
  #             -release \
  #             -confirm-license \
  #             -xplatform android-clang \
  #             -ssl \
  #             -openssl-runtime -I "${android-openssl}/ssl_3/include" \
  #             -L "${android-openssl}/ssl_3" \
  #             -android-ndk-host linux-x86_64 \
  #             -android-sdk $ANDROID_SDK_ROOT \
  #             -android-ndk $ANDROID_NDK_ROOT \
  #             -no-warnings-are-errors \
  #             -disable-rpath \
  #             -nomake tests \
  #             -nomake examples \
  #             -submodules ${ builtins.concatStringSep "," submodules }
  # '';
              # -skip qttranslations \
              # -skip qtserialport \
              # -skip qtwebengine \
              # -skip qt3d \
              # -skip qtwayland \
              # -skip qtpurchasing \
              # -skip qtvirtualkeyboard \
              # -skip qtspeech \

  # TODO use more cores
  # make -j$NIX_BUILD_CORES
  # buildPhase = ''
  #   make -j4
  # '';

  # installPhase = ''
  #   make install
  # '';

  # phases = [ "unpackPhase" ];

  # unpackPhase = ''
  #   ${lib.getExe p7zip} -o$out x $src
  # '';

  # qtPluginPrefix = "lib/qt-6/plugins";
  # qtQmlPrefix = "lib/qt-6/qml";
}
