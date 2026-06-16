package com.demo.barrier;

public class WithLoadBarrier {
    int data = 0;
    volatile boolean ready = false;

    public void producer() {
        data = 42;// (1)
        ready = true;// (2)
        // volatile 写
        // 这一行隐式插入 store barrier
        // barrier 保证 (1) 必须在 （2）之前完成
        // 并且 （1） r 写入已经从 store buffer 刷到 cache 
    }

    public void consumer() {
        if (ready) {// volatile 读
            // + load barrier
            // barrier 保证
            // invalid queue 已处理完
            // 之后读能够看到最新的数据

            int v = data; //
            System.out.println("data = " + data);
        }
    }
}
