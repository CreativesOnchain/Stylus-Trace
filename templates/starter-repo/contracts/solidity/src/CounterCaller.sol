// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

interface ICounter {
    function increment(uint256 amount) external returns (uint256);
    function get() external view returns (uint256);
}

contract CounterCaller {
    function bump(address counter, uint256 amount) external returns (uint256) {
        return ICounter(counter).increment(amount);
    }

    function read(address counter) external view returns (uint256) {
        return ICounter(counter).get();
    }
}
