/**
 * portaudio_backend.hpp — PortAudio output device backend.
 *
 * Opens the audio device, registers the realtime callback, and drives the
 * Mixer.  Used on Linux and Windows.  The macOS CoreAudio backend (Phase 2)
 * will implement the same interface via a platform #ifdef in engine.cpp.
 */

#pragma once

#include "mixer.hpp"

#include <portaudio.h>
#include <string>

namespace pdj {

/**
 * Result of backend initialisation.
 */
enum class BackendResult {
    Ok,
    PaInitError,      ///< Pa_Initialize() failed.
    DeviceNotFound,   ///< Requested device name not found.
    StreamOpenError,  ///< Pa_OpenStream() failed.
    StreamStartError, ///< Pa_StartStream() failed.
};

/**
 * PortAudio output backend.
 *
 * Lifecycle: construct → open() → (stream runs) → close() → destruct.
 */
class PortAudioBackend {
public:
    PortAudioBackend();
    ~PortAudioBackend();

    // Non-copyable.
    PortAudioBackend(const PortAudioBackend&) = delete;
    PortAudioBackend& operator=(const PortAudioBackend&) = delete;

    /**
     * Open the output device and start the audio stream.
     *
     * @param mixer        The mixer to call each callback.
     * @param sample_rate  Output sample rate (e.g. 44100).
     * @param buffer_size  Frames per callback (e.g. 128).
     * @param device_name  Empty string = system default.
     */
    BackendResult open(Mixer*       mixer,
                       uint32_t     sample_rate,
                       uint32_t     buffer_size,
                       const std::string& device_name);

    /** Stop the stream and release the device. */
    void close();

    /** True while the stream is running. */
    bool is_running() const;

    /** Human-readable description of the last PortAudio error. */
    std::string last_error() const;

private:
    /** Static callback forwarded to the instance method below. */
    static int pa_callback(const void* input, void* output,
                            unsigned long frames_per_buffer,
                            const PaStreamCallbackTimeInfo* time_info,
                            PaStreamCallbackFlags status_flags,
                            void* user_data);

    /** Instance callback — called by PortAudio on the RT thread. */
    int on_callback(float* output, uint32_t frames) noexcept;

    PaStream* stream_{nullptr};
    Mixer*    mixer_{nullptr};
    bool      pa_initialised_{false};
    PaError   last_pa_error_{paNoError};
};

} // namespace pdj
