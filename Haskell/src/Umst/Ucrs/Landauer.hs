{-# LANGUAGE OverloadedStrings #-}
module Umst.Ucrs.Landauer
  ( landauerCost
  , kB
  ) where

kB :: Double
kB = 1.380649e-23

landauerCost :: Double -> Double -> Double
landauerCost bits tempK
  | bits <= 0 || tempK <= 0 = 0
  | otherwise = kB * tempK * log 2 * bits
