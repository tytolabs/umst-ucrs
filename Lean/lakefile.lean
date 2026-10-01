-- SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
-- SPDX-License-Identifier: MIT

import Lake
open Lake DSL

package «umst-ucrs» where
  leanOptions := #[⟨`autoImplicit, false⟩]

require mathlib from git
  "https://github.com/leanprover-community/mathlib4.git" @ "v4.14.0"

-- Pinned public sibling: the commit whose CoordinationContract the runtime refines.
require «umst-formal» from git
  "https://github.com/tytolabs/umst-formal.git" @ "8f08d4a7cd4520da8a6acab6a66af4c34374c020" / "Lean"

/-!
  The runtime's laws are proved once, in umst-formal `CoordinationContract` (Lean, Coq, Agda, Haskell).
  `RuntimeContract` restates each law the runtime relies on and closes it with that theorem at the pinned commit,
  so a change upstream to one of those statements fails this build. A default target: `lake build` builds it.
-/
@[default_target]
lean_lib RuntimeContract where
  roots := #[`RuntimeContract]
  srcDir := "."
