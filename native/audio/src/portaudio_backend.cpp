/**
 * portaudio_backend.cpp — PortAudio output backend implementation.
 */

#include "portaudio_backend.hpp"

#include <cstring>

namespace pdj {

PortAudioBackend::PortAudioBackend() = default;

PortAudioBackend::~PortAudioBackend() {
    close();
    if (pa_initialised_) Pa_Terminate();
}

// ---------------------------------------------------------------------------
// open
// ---------------------------------------------------------------------------

BackendResult PortAudioBackend::open(Mixer*              mixer,
                                      uint32_t            sample_rate,
                                      uint32_t            buffer_size,
                                      const std::string&  device_name) {
    mixer_ = mixer;

    // Initialise PortAudio once.
    PaError err = Pa_Initialize();
    if (err != paNoError) {
        last_pa_error_ = err;
        return BackendResult::PaInitError;
    }
    pa_initialised_ = true;

    // Resolve the output device.
    PaDeviceIndex device_idx = Pa_GetDefaultOutputDevice();
    if (device_idx == paNoDevice) {
        last_pa_error_ = paDeviceUnavailable;
        return BackendResult::DeviceNotFound;
    }

    if (!device_name.empty()) {
        const int n = Pa_GetDeviceCount();
        bool found = false;
        for (int i = 0; i < n; ++i) {
            const PaDeviceInfo* di = Pa_GetDeviceInfo(i);
            if (di && std::string(di->name).find(device_name) != std::string::npos
                   && di->maxOutputChannels >= 2) {
                device_idx = i;
                found = true;
                break;
            }
        }
        if (!found) {
            last_pa_error_ = paDeviceUnavailable;
            return BackendResult::DeviceNotFound;
        }
    }

    const PaDeviceInfo* dev_info = Pa_GetDeviceInfo(device_idx);

    // Output stream parameters.
    PaStreamParameters out_params{};
    out_params.device                    = device_idx;
    out_params.channelCount              = 2;
    out_params.sampleFormat              = paFloat32;
    out_params.suggestedLatency          = dev_info->defaultLowOutputLatency;
    out_params.hostApiSpecificStreamInfo = nullptr;

    err = Pa_OpenStream(
        &stream_,
        nullptr,              // no input
        &out_params,
        static_cast<double>(sample_rate),
        static_cast<unsigned long>(buffer_size),
        paClipOff,            // we handle limiting ourselves
        &PortAudioBackend::pa_callback,
        this);

    if (err != paNoError) {
        last_pa_error_ = err;
        return BackendResult::StreamOpenError;
    }

    err = Pa_StartStream(stream_);
    if (err != paNoError) {
        last_pa_error_ = err;
        Pa_CloseStream(stream_);
        stream_ = nullptr;
        return BackendResult::StreamStartError;
    }

    return BackendResult::Ok;
}

// ---------------------------------------------------------------------------
// close
// ---------------------------------------------------------------------------

void PortAudioBackend::close() {
    if (stream_) {
        Pa_StopStream(stream_);
        Pa_CloseStream(stream_);
        stream_ = nullptr;
    }
}

bool PortAudioBackend::is_running() const {
    return stream_ && Pa_IsStreamActive(stream_) == 1;
}

std::string PortAudioBackend::last_error() const {
    return Pa_GetErrorText(last_pa_error_);
}

// ---------------------------------------------------------------------------
// Audio callback
// ---------------------------------------------------------------------------

int PortAudioBackend::pa_callback(const void* /*input*/,
                                   void*        output,
                                   unsigned long frames_per_buffer,
                                   const PaStreamCallbackTimeInfo* /*time*/,
                                   PaStreamCallbackFlags /*flags*/,
                                   void* user_data) {
    auto* self = static_cast<PortAudioBackend*>(user_data);
    return self->on_callback(static_cast<float*>(output),
                              static_cast<uint32_t>(frames_per_buffer));
}

int PortAudioBackend::on_callback(float* output, uint32_t frames) noexcept {
    // Clear buffer first (silence if mixer has nothing).
    std::memset(output, 0, static_cast<std::size_t>(frames) * 2 * sizeof(float));

    if (mixer_) {
        mixer_->process_block(output, frames);
    }

    // paContinue keeps the stream running.
    return paContinue;
}

} // namespace pdj
