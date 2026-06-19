{-# LANGUAGE OverloadedStrings #-}
module Umst.Ucrs.Credit
  ( PeerCredit (..)
  , bestPeer
  , recordSync
  ) where

import Data.Ord (comparing)

data PeerCredit = PeerCredit
  { peerId :: Int
  , creditBits :: Double
  , accuracyScore :: Double
  , syncCount :: Int
  }
  deriving (Eq, Show)

bestPeer :: [PeerCredit] -> Maybe Int
bestPeer peers =
  case filter ((> 0.1) . accuracyScore) peers of
    [] -> Nothing
    healthy ->
      Just . peerId $
        maximumBy (comparing creditBits) healthy
  where
    maximumBy f (x : xs) = foldl (\a b -> if f a b == GT then a else b) x xs
    maximumBy _ [] = error "unreachable"

recordSync :: PeerCredit -> Double -> Bool -> PeerCredit
recordSync p bits improved =
  let n = syncCount p + 1
   in if improved
        then p
          { creditBits = creditBits p + bits
          , accuracyScore = 0.9 * accuracyScore p + 0.1
          , syncCount = n
          }
        else p
          { creditBits = creditBits p - 2 * bits
          , accuracyScore = accuracyScore p * 0.9
          , syncCount = n
          }
