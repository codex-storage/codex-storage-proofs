
import std/os
import std/strutils
import std/sha1
import std/macros

const
  currentDir = currentSourcePath().parentDir()
  libPath* = currentDir/"target"/"release"/"libcodex_storage_proofs.a"

static:
  let cmd = "cargo build --release"
  hint "\nBuilding codex-storage-proofs: " & cmd
  let (output, exitCode) = gorgeEx cmd
  for ln in output.splitLines():
    hint("cargo> " & ln)
  if exitCode != 0:
    raise (ref Defect)(msg: "Failed to build codex-storage-proofs")

