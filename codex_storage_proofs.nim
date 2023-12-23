
import std/os
import std/strutils
import std/sha1
import std/macros

const
  currentDir = currentSourcePath().parentDir()
  libDir* = currentDir/"target"/"release"
  libPath* = libDir/"libcodex_storage_proofs.a"

static:
  let cmd = "cargo build --release"
  warning "\nBuilding codex-storage-proofs: " & cmd
  let (output, exitCode) = gorgeEx cmd
  for ln in output.splitLines():
    warning("cargo> " & ln)
  if exitCode != 0:
    raise (ref Defect)(msg: "Failed to build codex-storage-proofs")


{.passl: "-lcodex_storage_proofs" & " -L" & libDir.}

import codex_proofs_ffi
export codex_proofs_ffi

when isMainModule:
  init_proof_ctx()
