/**
 * ringbuffer_test.cpp — Unit tests for the lock-free SPSC ring buffer.
 */

#include "ringbuffer.hpp"

#include <gtest/gtest.h>
#include <thread>
#include <vector>

using pdj::RingBuffer;

TEST(RingBuffer, EmptyAtStart) {
    RingBuffer<float, 64> rb;
    EXPECT_EQ(rb.read_available(), 0u);
    EXPECT_EQ(rb.write_available(), 64u);
    EXPECT_TRUE(rb.empty());
}

TEST(RingBuffer, PushAndPopRoundTrip) {
    RingBuffer<float, 64> rb;
    float in[] = {1, 2, 3, 4, 5};
    EXPECT_EQ(rb.push(in, 5), 5u);
    EXPECT_EQ(rb.read_available(), 5u);

    float out[5] = {0};
    EXPECT_EQ(rb.pop(out, 5), 5u);
    for (int i = 0; i < 5; ++i) EXPECT_FLOAT_EQ(out[i], in[i]);
    EXPECT_TRUE(rb.empty());
}

TEST(RingBuffer, FullStopsWriting) {
    RingBuffer<float, 8> rb;
    float in[10] = {1, 2, 3, 4, 5, 6, 7, 8, 9, 10};
    EXPECT_EQ(rb.push(in, 10), 8u);   // capped at capacity
    EXPECT_EQ(rb.write_available(), 0u);
}

TEST(RingBuffer, EmptyStopsReading) {
    RingBuffer<float, 8> rb;
    float out[4];
    EXPECT_EQ(rb.pop(out, 4), 0u);
}

TEST(RingBuffer, WrapAround) {
    RingBuffer<float, 8> rb;
    float in1[6] = {1, 2, 3, 4, 5, 6};
    rb.push(in1, 6);
    float out1[4];
    rb.pop(out1, 4);  // free up space at the start

    // Now push something that wraps.
    float in2[5] = {7, 8, 9, 10, 11};
    EXPECT_EQ(rb.push(in2, 5), 5u);

    float out2[7];
    EXPECT_EQ(rb.pop(out2, 7), 7u);
    EXPECT_FLOAT_EQ(out2[0], 5);  // remaining from in1
    EXPECT_FLOAT_EQ(out2[1], 6);
    EXPECT_FLOAT_EQ(out2[2], 7);
    EXPECT_FLOAT_EQ(out2[6], 11);
}

TEST(RingBuffer, ResetClears) {
    RingBuffer<float, 8> rb;
    float in[4] = {1, 2, 3, 4};
    rb.push(in, 4);
    rb.reset();
    EXPECT_TRUE(rb.empty());
}

TEST(RingBuffer, ConcurrentProducerConsumer) {
    // 64 K capacity, push 1 M floats, consumer reads all.
    constexpr std::size_t CAP = 1 << 16;
    auto rb = std::make_unique<RingBuffer<float, CAP>>();

    constexpr std::size_t TOTAL = 1'000'000;
    std::atomic<bool> done{false};

    std::thread producer([&] {
        std::size_t written = 0;
        while (written < TOTAL) {
            float buf[256];
            for (int i = 0; i < 256; ++i) buf[i] = static_cast<float>(written + i);
            const std::size_t n = std::min<std::size_t>(256, TOTAL - written);
            std::size_t got = rb->push(buf, n);
            written += got;
            if (got == 0) std::this_thread::yield();
        }
        done.store(true);
    });

    std::thread consumer([&] {
        std::size_t read = 0;
        float buf[512];
        while (read < TOTAL) {
            std::size_t got = rb->pop(buf, 512);
            for (std::size_t i = 0; i < got; ++i) {
                ASSERT_FLOAT_EQ(buf[i], static_cast<float>(read + i));
            }
            read += got;
            if (got == 0 && !done.load()) std::this_thread::yield();
        }
    });

    producer.join();
    consumer.join();
}
