{-# LANGUAGE OverloadedStrings #-}
module Umst.Ucrs.Gate
  ( GateVerdict (..)
  , gateCheck
  ) where

import Umst.Ucrs.Landauer (landauerCost)

data GateVerdict = Admit | Reject
  deriving (Eq, Show)

gateCheck :: Double -> Double -> Double -> GateVerdict
gateCheck desyncEnergyJ budgetJ bitsToResolve
  | desyncEnergyJ <= 0 = Reject
  | landauerCost bitsToResolve 300.0 > budgetJ = Reject
  | otherwise = Admit
