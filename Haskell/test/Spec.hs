{-# LANGUAGE OverloadedStrings #-}
module Main where

import Test.Tasty
import Test.Tasty.QuickCheck
import Control.Monad (forM)
import Umst.Ucrs.Credit
import Umst.Ucrs.Gate
import Umst.Ucrs.Landauer

main :: IO ()
main = defaultMain tests

tests :: TestTree
tests = testGroup "umst-ucrs properties"
  [ testProperty "greedy selects highest credit" prop_greedy_selects_highest_credit
  , testProperty "byzantine credit drops" prop_byzantine_credit_drops
  , testProperty "gate rejects over budget" prop_gate_rejects_over_budget
  , testProperty "landauer monotonic in bits" prop_landauer_monotonic_in_bits
  , testProperty "gate admits within budget" prop_gate_admits_within_budget
  ]

-- | Property 1: greedy peer selection picks highest credit among healthy peers.
prop_greedy_selects_highest_credit :: Property
prop_greedy_selects_highest_credit =
  forAll genPeers $ \peers ->
    case bestPeer peers of
      Nothing -> length (filter ((> 0.1) . accuracyScore) peers) == 0
      Just pid ->
        let chosen = head [p | p <- peers, peerId p == pid]
            healthy = filter ((> 0.1) . accuracyScore) peers
        in all (\p -> creditBits chosen >= creditBits p) healthy

-- | Property 2: bad sync (Byzantine signal) reduces credit.
prop_byzantine_credit_drops :: Property
prop_byzantine_credit_drops =
  forAll (choose (1.0, 10.0)) $ \bits ->
    let p0 = PeerCredit 1 5.0 1.0 0
        p1 = recordSync p0 bits True
        p2 = recordSync p1 bits False
    in creditBits p2 < creditBits p1

-- | Property 3: gate rejects when sync cost exceeds budget.
prop_gate_rejects_over_budget :: Property
prop_gate_rejects_over_budget =
  forAll ((,) <$> choose (1.0, 5.0) <*> choose (6.0, 20.0)) $ \(budgetBits, resolveBits) ->
    let budgetJ = landauerCost budgetBits 300.0
    in gateCheck 5.0 budgetJ resolveBits == Reject

-- | Property 4: Landauer cost is monotonic in bits at fixed temperature.
prop_landauer_monotonic_in_bits :: Property
prop_landauer_monotonic_in_bits =
  forAll ((,) <$> choose (0.1, 5.0) <*> choose (0.1, 5.0)) $ \(b1, b2) ->
    let t = 300.0
        c1 = landauerCost b1 t
        c2 = landauerCost b2 t
    in if b1 <= b2 then c1 <= c2 else c2 <= c1

-- | Property 5: gate admits when cost is within budget and desync energy.
prop_gate_admits_within_budget :: Property
prop_gate_admits_within_budget =
  forAll (choose (1.0, 10.0)) $ \budget ->
    let desync = budget * 2
        cost = budget * 0.5
    in gateCheck desync budget cost == Admit

genPeers :: Gen [PeerCredit]
genPeers =
  forM [1 .. 4] $ \pid -> do
    credit <- choose (0.0, 20.0)
    acc <- choose (0.2, 1.0)
    pure (PeerCredit pid credit acc 0)
