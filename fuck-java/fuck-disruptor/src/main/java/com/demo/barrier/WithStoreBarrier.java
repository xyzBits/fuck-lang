package com.demo.barrier;

public class WithoutBarrier {
    int data = 0;
    boolean ready = false;

    public void producer() {
        data = 42;
        ready = true;
    }

    public void consumer() {
        if (ready) {
            System.out.println("data = " + data);
        }
    }
}
