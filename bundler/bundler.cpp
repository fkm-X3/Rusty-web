#include <cstdlib>
#include <direct.h> // _getcwd, _chdir
#include <iostream>
#include <string>
#include <windows.h>

// Helper to run command
bool run_command(const std::string &cmd) {
  std::cout << "[Bundler] Running: " << cmd << std::endl;
  int ret = std::system(cmd.c_str());
  if (ret != 0) {
    std::cerr << "[Bundler] Command failed with exit code " << ret << std::endl;
    return false;
  }
  return true;
}

std::string get_current_dir() {
  char buff[FILENAME_MAX];
  if (_getcwd(buff, FILENAME_MAX))
    return std::string(buff);
  return "";
}

bool dir_exists(const std::string &path) {
  DWORD ftyp = GetFileAttributesA(path.c_str());
  if (ftyp == INVALID_FILE_ATTRIBUTES)
    return false;
  return (ftyp & FILE_ATTRIBUTE_DIRECTORY);
}

bool file_exists(const std::string &path) {
  DWORD ftyp = GetFileAttributesA(path.c_str());
  if (ftyp == INVALID_FILE_ATTRIBUTES)
    return false;
  return !(ftyp & FILE_ATTRIBUTE_DIRECTORY);
}

int main() {
  std::cout << "===========================================" << std::endl;
  std::cout << "   Rusty-Web C++ Bundler (Portable)" << std::endl;
  std::cout << "===========================================" << std::endl;

  std::string root_dir = get_current_dir();

  // Check if we need to go up (if in bundler subdir)
  if (!dir_exists(root_dir + "\\rusty_web_core")) {
    // Try up one level
    if (dir_exists(root_dir + "\\..\\rusty_web_core")) {
      if (_chdir("..") == 0) {
        root_dir = get_current_dir();
        std::cout << "Changed working directory to root: " << root_dir
                  << std::endl;
      }
    }
  }

  if (!dir_exists(root_dir + "\\rusty_web_core")) {
    std::cerr << "Error: Could not find 'rusty_web_core'. Make sure you run "
                 "this from the project root or bundler directory."
              << std::endl;
    return 1;
  }

  // 1. Build Rust
  std::cout << "\n[Step 1/3] Building Rust Core (Static Lib)..." << std::endl;
  _putenv("RUSTFLAGS=-C target-feature=+crt-static");
  if (!run_command("cd rusty_web_core && cargo build --release -j 1"))
    return 1;

  // 2. Verify Artifacts
  std::string target_dir = root_dir + "\\rusty_web_core\\target\\release";
  std::string rust_lib = target_dir + "\\rusty_web_core.lib";

  if (!file_exists(rust_lib)) {
    std::cerr << "Error: " << rust_lib << " is missing. Build likely failed."
              << std::endl;
    return 1;
  }

  // Copy WebView2Loader.dll if it exists (std::filesystem free)
  std::string dll_name = "WebView2Loader.dll";
  std::string src_dll = target_dir + "\\" + dll_name;
  std::string dst_dll = root_dir + "\\" + dll_name;

  if (file_exists(src_dll)) {
    std::cout << "Copying " << dll_name << " to output directory..."
              << std::endl;
    if (CopyFileA(src_dll.c_str(), dst_dll.c_str(), FALSE)) {
      std::cout << "Success copying DLL." << std::endl;
    } else {
      std::cerr << "Warning: Failed to copy DLL." << std::endl;
    }
  }

  // 3. Compiling Final Executable
  std::cout << "\n[Step 3/3] Compiling C++ Launcher..." << std::endl;

  // Quote paths just in case (though command prompt quoting can be tricky, cl
  // handles straightforward paths well) We use \" for paths with spaces.
  std::string cl_cmd =
      "cl /nologo /O2 /MT /EHsc packager\\main.cpp /Fe:Browser.exe ";

  // /LIBPATH must be quote wrapped
  cl_cmd += "/link /LIBPATH:\"" + target_dir + "\" rusty_web_core.lib ";

  // Add libs
  cl_cmd += "user32.lib shell32.lib ole32.lib oleaut32.lib advapi32.lib "
            "gdi32.lib shlwapi.lib dwmapi.lib uxtheme.lib bcrypt.lib imm32.lib "
            "ws2_32.lib crypt32.lib propsys.lib ntdll.lib";

  if (!run_command(cl_cmd)) {
    std::cerr << "C++ Compilation failed." << std::endl;
    return 1;
  }

  std::cout << "\n===========================================" << std::endl;
  std::cout << "Build SUCCESS!" << std::endl;
  std::cout << "Output: " << root_dir << "\\Browser.exe" << std::endl;
  std::cout << "===========================================" << std::endl;
  return 0;
}
