package com.demo.async;

import com.google.common.util.concurrent.*;

import java.util.concurrent.CompletableFuture;
import java.util.concurrent.ExecutorService;
import java.util.concurrent.Executors;
import java.util.concurrent.Future;

public class AsyncDiff {


    static void main() {
        AsyncDiff asyncDiff = new AsyncDiff();
        asyncDiff.demoByRunnable();
        asyncDiff.demoByFuture();

        asyncDiff.demoByListenableFuture();

        asyncDiff.demoByCompletableFuture();

        asyncDiff.demoByJdk21();

    }


    private void demoByJdk21() {
        try(var executor = Executors.newVirtualThreadPerTaskExecutor()) {
            IO.println("demoByJdk21 start....");

            executor.submit(() -> {


                try {
                    String user = fetchUsers("123");

                    String orders = fetchOrders(user);

                    IO.println("demoByJdk21 orders = " + orders);

                } catch (Exception e) {
                    throw new RuntimeException(e);
                }

            });

        } catch (Exception e) {
            IO.println("demoByJdk21 error: " + e);

        }
    }


    private String fetchUsers(String id) throws Exception {
        Thread.sleep(1000);
        return "User_" + id;
    }

    private String fetchOrders(String user) throws Exception {
        Thread.sleep(1000);
        return user + "_Orders";
    }


    private void demoByRunnable() {
        new Thread(new Runnable() {
            @Override
            public void run() {
                try {

                    IO.println("demoByRunnable start....");
                    var user = fetchUsers("123");

                    new Thread(new Runnable() {
                        @Override
                        public void run() {
                            try {
                                String orders = fetchOrders(user);
                                IO.println("demoByRunnable orders = " + orders);
                            } catch (Exception e) {
                                IO.println("demoByRunnable fetchOrders error: " + e);
                            }
                        }
                    }).start();

                } catch (Exception e) {
                    IO.println("demoByRunnable fetchUser error: " + e);
                }
            }
        }).start();
    }


    private void demoByFuture() {
        try (ExecutorService executor = Executors.newFixedThreadPool(10)) {
            System.out.println("demoByFuture stat....");

            Future<String> userFuture = executor.submit(() -> fetchOrders("123"));

            String user = userFuture.get();

            Future<String> ordersFuture = executor.submit(() -> fetchOrders(user));

            String orders = ordersFuture.get();
            System.out.println("demoByFuture orders = " + orders);
        } catch (Exception e) {
            System.out.println("demyByFuture error: " + e);
        }
    }


    private void demoByListenableFuture() {


        System.out.println("demoByListenableFuture start...");
        ListeningExecutorService listeningExecutor = MoreExecutors.listeningDecorator(Executors.newFixedThreadPool(10));

        ListenableFuture<String> userFuture = listeningExecutor.submit(() -> fetchUsers("123"));

        Futures.addCallback(userFuture, new FutureCallback<String>() {
            @Override
            public void onSuccess(String user) {
                ListenableFuture<String> ordersFuture = listeningExecutor.submit(() -> fetchOrders(user));

                Futures.addCallback(ordersFuture, new FutureCallback<String>() {
                    @Override
                    public void onSuccess(String orders) {
                        System.out.println("demoByListenableFuture orders = " + orders);
                    }

                    @Override
                    public void onFailure(Throwable t) {

                    }
                }, listeningExecutor);
            }

            @Override
            public void onFailure(Throwable t) {

            }
        }, listeningExecutor);
    }


    private void demoByCompletableFuture() {
        System.out.println("demoByCompletableFuture start...");

        ExecutorService executor = Executors.newFixedThreadPool(10);

        CompletableFuture.supplyAsync(() -> {
                    try {
                        return fetchUsers("123");
                    } catch (Exception e) {
                        throw new RuntimeException(e);
                    }
                }, executor)
                .thenComposeAsync(user -> CompletableFuture.supplyAsync(() -> {
                    try {
                        String orders = fetchOrders(user);
                        System.out.println("demoByCompletableFuture orders = " + orders);
                        return orders;
                    } catch (Exception e) {
                        throw new RuntimeException(e);
                    }
                }, executor), executor);
    }
}
