
import std/os

const
  currentDir = currentSourcePath().parentDir()
  lib_path* = currentDir/"target"/"release"/"libcodex_storage_proofs.a"

static:
  echo "\n==== CODEX:STORAGE:PROOFS: "
  echo "cwd: ", currentDir
  echo "lib_path: ", lib_path
  # echo "pwd: ", projectDir()
  # echo "cwd: ", getCurrentDir()
  let cmd = "pwd && cargo build --release"
  echo "\nBuilding codex-storage-proofs: " & cmd
  let (output, exitCode) = gorgeEx cmd
  echo output
  if exitCode != 0:
    # discard gorge "rm -rf " & buildDir
    raise (ref Defect)(msg: "Failed to build codex-storage-proofs")

  echo "\n===="
