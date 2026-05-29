#include <cuda_runtime.h>

#include <algorithm>
#include <chrono>
#include <cmath>
#include <cstdint>
#include <cstdio>
#include <cstring>
#include <mutex>

extern "C" {

struct DhoolCudaDeviceInfo {
    int available;
    char name[256];
    int driver_version;
    int runtime_version;
    unsigned long long global_memory_bytes;
    int multiprocessor_count;
    int error_code;
    char error_message[512];
};

struct DhoolRenderReport {
    int status;
    float gpu_ms;
    float readback_ms;
    int error_code;
    char error_message[512];
};

struct DhoolRenderParams {
    uint32_t width;
    uint32_t height;
    uint32_t frame_index;
    uint32_t total_frames;
    uint64_t seed;
    float time_seconds;
    float duration_seconds;
    float fps;
    uint32_t quality;
    uint32_t field_width;
    uint32_t field_height;
    uint32_t substeps;
    float velocity_decay;
    float vorticity;
    float buoyancy;
    float turbulence;
    float swirl;
    float advection_dissipation;
    float plasma_emission_strength;
    float plasma_filament_density;
    float plasma_core_temperature;
    float plasma_edge_wrapping;
    float plasma_color_core[3];
    float plasma_color_edge[3];
    uint32_t particles_count;
    float particles_size_min;
    float particles_size_max;
    float particles_lifetime;
    float particles_flow_coupling;
    float particles_ember_brightness;
    float particles_gravity;
    float particles_drag;
    float particles_thermal_lift;
    float particles_jitter;
    float smoke_density;
    float smoke_scattering;
    float smoke_absorption;
    float smoke_color_cool[3];
    float smoke_color_warm[3];
    float bloom_threshold;
    float bloom_strength;
    float bloom_radius;
    uint32_t bloom_pyramid_levels;
    float grade_exposure;
    float grade_contrast;
    float grade_saturation;
    float grade_black_point;
    float grade_vignette;
    float grade_grain;
};

}

struct BloomView {
    const float4* levels[8];
    int widths[8];
    int heights[8];
    int level_count;
};

struct Particle {
    float2 pos;
    float2 vel;
    float age;
    float heat;
    float size;
    float alpha;
};

static_assert(sizeof(unsigned long long) == sizeof(uint64_t), "unexpected uint64 size");

struct DeviceBuffer {
    void* ptr = nullptr;
    size_t capacity = 0;
};

struct HostBuffer {
    void* ptr = nullptr;
    size_t capacity = 0;
};

struct RenderCache {
    DeviceBuffer hdr;
    DeviceBuffer ldr;
    DeviceBuffer velocity;
    DeviceBuffer density;
    DeviceBuffer temperature;
    DeviceBuffer particles;
    DeviceBuffer bloom_levels[8];
    HostBuffer readback;
    cudaEvent_t gpu_start {};
    cudaEvent_t gpu_stop {};
    cudaEvent_t copy_start {};
    cudaEvent_t copy_stop {};
    bool events_ready = false;
};

std::mutex g_render_cache_mutex;
RenderCache g_render_cache;

__device__ float clamp01(float v) {
    return fminf(1.0f, fmaxf(0.0f, v));
}

__device__ float saturate(float v) {
    return clamp01(v);
}

__device__ float2 add2(float2 a, float2 b) {
    return make_float2(a.x + b.x, a.y + b.y);
}

__device__ float2 sub2(float2 a, float2 b) {
    return make_float2(a.x - b.x, a.y - b.y);
}

__device__ float2 mul2(float2 a, float s) {
    return make_float2(a.x * s, a.y * s);
}

__device__ float dot2(float2 a, float2 b) {
    return a.x * b.x + a.y * b.y;
}

__device__ float len2(float2 a) {
    return sqrtf(dot2(a, a));
}

__device__ float2 normalize2(float2 v) {
    float l = len2(v);
    return l > 1.0e-6f ? make_float2(v.x / l, v.y / l) : make_float2(0.0f, 1.0f);
}

__device__ float3 add3(float3 a, float3 b) {
    return make_float3(a.x + b.x, a.y + b.y, a.z + b.z);
}

__device__ float3 mul3(float3 a, float s) {
    return make_float3(a.x * s, a.y * s, a.z * s);
}

__device__ float3 mix3(float3 a, float3 b, float t) {
    return make_float3(
        a.x + (b.x - a.x) * t,
        a.y + (b.y - a.y) * t,
        a.z + (b.z - a.z) * t
    );
}

__device__ uint64_t splitmix64(uint64_t x) {
    x += 0x9e3779b97f4a7c15ULL;
    uint64_t z = x;
    z = (z ^ (z >> 30)) * 0xbf58476d1ce4e5b9ULL;
    z = (z ^ (z >> 27)) * 0x94d049bb133111ebULL;
    return z ^ (z >> 31);
}

__device__ float hash_u64_to_unit(uint64_t x) {
    return static_cast<float>((splitmix64(x) >> 40) & 0xFFFFFFULL) / 16777216.0f;
}

__device__ float hash_i2(int x, int y, uint64_t seed) {
    uint64_t h = static_cast<uint64_t>(x) * 0x9e3779b185ebca87ULL;
    h ^= static_cast<uint64_t>(y) * 0xc2b2ae3d27d4eb4fULL;
    h ^= seed;
    return hash_u64_to_unit(h);
}

__device__ float smooth(float t) {
    return t * t * (3.0f - 2.0f * t);
}

__device__ float noise2(float2 p, uint64_t seed) {
    int ix = static_cast<int>(floorf(p.x));
    int iy = static_cast<int>(floorf(p.y));
    float fx = p.x - floorf(p.x);
    float fy = p.y - floorf(p.y);
    float ux = smooth(fx);
    float uy = smooth(fy);
    float a = hash_i2(ix, iy, seed);
    float b = hash_i2(ix + 1, iy, seed);
    float c = hash_i2(ix, iy + 1, seed);
    float d = hash_i2(ix + 1, iy + 1, seed);
    float ab = a + (b - a) * ux;
    float cd = c + (d - c) * ux;
    return ab + (cd - ab) * uy;
}

__device__ float fbm(float2 p, uint64_t seed) {
    float sum = 0.0f;
    float amp = 0.52f;
    float freq = 1.0f;
    for (int i = 0; i < 5; ++i) {
        sum += amp * noise2(make_float2(p.x * freq, p.y * freq), seed + static_cast<uint64_t>(i) * 131ULL);
        freq *= 2.03f;
        amp *= 0.49f;
        p = make_float2(p.x * 1.17f + p.y * 0.23f, -p.x * 0.19f + p.y * 1.11f);
    }
    return sum;
}

__device__ float ridge(float v) {
    return 1.0f - fabsf(v * 2.0f - 1.0f);
}

__device__ float line_gaussian(float distance, float width) {
    return expf(-(distance * distance) * width);
}

__device__ float2 p_to_uv(float2 p, float aspect) {
    return make_float2(p.x / aspect + 0.5f, 0.5f - p.y);
}

__device__ float2 uv_to_p(float2 uv, float aspect) {
    return make_float2((uv.x - 0.5f) * aspect, 0.5f - uv.y);
}

__device__ float2 field_flow(float2 p, float t, uint64_t seed, const DhoolRenderParams& params) {
    float n1 = fbm(make_float2(p.x * 1.6f + t * 0.10f, p.y * 1.4f - t * 0.05f), seed + 17ULL);
    float n2 = fbm(make_float2(p.x * 2.4f - t * 0.07f, p.y * 2.1f + t * 0.09f), seed + 53ULL);
    float angle = (n1 * 2.0f - 1.0f) * 6.2831853f * (0.65f + params.turbulence * 0.30f);
    angle += (p.x * 1.7f - p.y * 0.8f + n2) * params.swirl;
    float2 curl = make_float2(cosf(angle), sinf(angle));
    float2 lift = make_float2(0.18f * sinf(p.y * 5.0f + t), 0.72f + params.buoyancy);
    float2 orbital = normalize2(make_float2(-p.y - 0.14f, p.x + 0.18f));
    return add2(add2(mul2(curl, params.vorticity), mul2(lift, 0.42f)), mul2(orbital, 0.25f * params.swirl));
}

__device__ float plasma_mask(float2 p, float t, uint64_t seed, const DhoolRenderParams& params) {
    float aspect = static_cast<float>(params.width) / static_cast<float>(params.height);
    float2 flow = field_flow(p, t, seed, params);
    float2 warped = add2(p, mul2(flow, 0.052f + params.plasma_edge_wrapping * 0.030f));
    warped.x += (fbm(make_float2(p.x * 2.0f, p.y * 2.0f + t * 0.045f), seed + 71ULL) - 0.5f) * 0.15f;
    warped.y += (fbm(make_float2(p.x * 2.8f - t * 0.060f, p.y * 2.0f), seed + 73ULL) - 0.5f) * 0.12f;

    float x = warped.x / aspect;
    float y = warped.y;
    float left_curve = 0.080f - 0.54f * x - 0.050f * x * x + 0.045f * sinf(10.0f * x + t * 0.16f);
    float right_curve = -0.020f + 0.58f * (x - 0.02f) + 0.078f * sinf(8.4f * x - t * 0.14f);
    float lower_curve = -0.350f + 0.26f * (x + 0.08f) + 0.040f * sinf(12.0f * x + t * 0.12f);
    float far_curve = -0.205f + 0.11f * sinf(5.5f * x - 0.7f) + 0.11f * x;
    left_curve += (fbm(make_float2(x * 5.0f + t * 0.040f, 1.3f), seed + 81ULL) - 0.5f) * 0.045f;
    right_curve += (fbm(make_float2(x * 5.5f - t * 0.034f, 4.7f), seed + 83ULL) - 0.5f) * 0.050f;
    lower_curve += (fbm(make_float2(x * 4.8f + t * 0.028f, 8.2f), seed + 85ULL) - 0.5f) * 0.040f;
    far_curve += (fbm(make_float2(x * 3.2f - t * 0.020f, 6.1f), seed + 87ULL) - 0.5f) * 0.080f;

    float left_gate = smooth(saturate((0.24f - x) * 3.2f)) * smooth(saturate((x + 0.95f) * 2.5f));
    float right_gate = smooth(saturate((x + 0.02f) * 3.5f)) * smooth(saturate((0.96f - x) * 2.1f));
    float lower_gate = smooth(saturate((x + 0.16f) * 3.0f)) * smooth(saturate((-0.04f - y) * 3.1f));
    float far_gate = smooth(saturate((x + 0.65f) * 2.0f)) * smooth(saturate((0.75f - x) * 2.0f));
    float d_left = fabsf(y - left_curve);
    float d_right = fabsf(y - right_curve);
    float d_lower = fabsf(y - lower_curve);
    float d_far = fabsf(y - far_curve);
    float core_width = 900.0f + params.plasma_filament_density * 450.0f;
    float sheath_width = 52.0f + params.plasma_filament_density * 14.0f;
    float core = line_gaussian(d_left, core_width) * left_gate * 0.95f;
    core += line_gaussian(d_right, core_width * 0.92f) * right_gate * 1.05f;
    core += line_gaussian(d_lower, core_width * 0.82f) * lower_gate * 0.86f;
    core += line_gaussian(d_far, core_width * 0.38f) * far_gate * 0.32f;
    float sheath = line_gaussian(d_left, sheath_width) * left_gate * 0.80f;
    sheath += line_gaussian(d_right, sheath_width) * right_gate * 0.90f;
    sheath += line_gaussian(d_lower, sheath_width * 0.82f) * lower_gate * 0.72f;
    sheath += line_gaussian(d_far, sheath_width * 0.52f) * far_gate * 0.42f;

    float filament_wave = sinf((warped.x * 18.0f + warped.y * 8.5f + t * 0.20f) * (1.2f + params.plasma_filament_density));
    filament_wave += 0.7f * sinf((warped.x * -23.0f + warped.y * 13.0f - t * 0.16f) * (1.0f + params.plasma_filament_density * 0.55f));
    filament_wave += 0.35f * sinf((warped.x * 36.0f - warped.y * 20.0f + t * 0.10f));
    float filament = powf(saturate(ridge(0.5f + 0.18f * filament_wave)), 7.0f);
    float fine = powf(saturate(fbm(make_float2(warped.x * 30.0f + t * 0.12f, warped.y * 30.0f - t * 0.07f), seed + 101ULL)), 3.2f);
    float broken = smooth(saturate((fbm(make_float2(warped.x * 5.0f - t * 0.025f, warped.y * 4.2f + t * 0.020f), seed + 107ULL) - 0.30f) * 2.9f));
    float void_mask = expf(-(x * x * 18.0f + (y - 0.18f) * (y - 0.18f) * 34.0f));
    float source = core * (0.40f + filament * fine * 3.2f) * (0.58f + broken * 0.42f);
    source += sheath * (0.14f + fine * 0.22f + broken * 0.20f);
    return source * (1.0f - 0.88f * void_mask);
}

__device__ float terrain_base_y(float x) {
    float left_mass = 0.190f * expf(-((x + 0.72f) * (x + 0.72f)) / 0.045f);
    left_mass += 0.100f * expf(-((x + 0.44f) * (x + 0.44f)) / 0.120f);
    float right_mass = 0.235f * expf(-((x - 0.58f) * (x - 0.58f)) / 0.035f);
    right_mass += 0.130f * expf(-((x - 0.82f) * (x - 0.82f)) / 0.055f);
    float center_dip = -0.125f * expf(-(x * x) / 0.165f);
    float terrace = 0.035f * smooth(saturate((x - 0.12f) * 2.4f));
    return -0.420f + left_mass + right_mass + center_dip + terrace + 0.030f * sinf(x * 10.0f + 1.1f);
}

__device__ float terrain_ridge_y(float2 p, float t, uint64_t seed) {
    float x = p.x;
    float rock_noise = (fbm(make_float2(x * 5.2f + t * 0.006f, 0.5f), seed + 223ULL) - 0.5f) * 0.080f;
    rock_noise += (fbm(make_float2(x * 13.0f - t * 0.004f, 7.1f), seed + 227ULL) - 0.5f) * 0.030f;
    return terrain_base_y(x) + rock_noise;
}

__device__ float terrain_mask(float2 p, float t, uint64_t seed) {
    float ridge_line = terrain_ridge_y(p, t, seed);
    return smooth(saturate((ridge_line - p.y) * 28.0f));
}

__device__ float terrain_rim(float2 p, float t, uint64_t seed) {
    float ridge_line = terrain_ridge_y(p, t, seed);
    float edge = expf(-fabsf(p.y - ridge_line) * 58.0f);
    float upper = smooth(saturate((p.y - ridge_line + 0.070f) * 11.0f));
    float broken = 0.65f + 0.35f * fbm(make_float2(p.x * 18.0f + t * 0.012f, p.y * 11.0f), seed + 229ULL);
    return edge * upper * broken;
}

__device__ float smoke_layer(float2 p, float t, uint64_t seed, const DhoolRenderParams& params, float plasma) {
    float2 flow = field_flow(p, t, seed + 303ULL, params);
    float2 advected = add2(p, mul2(flow, -0.26f * params.advection_dissipation));
    advected.y -= t * 0.018f;
    float broad = fbm(make_float2(advected.x * 1.05f, advected.y * 0.95f), seed + 307ULL);
    float sheet = fbm(make_float2(advected.x * 3.0f + broad * 0.9f, advected.y * 2.1f - t * 0.030f), seed + 311ULL);
    float veil = fbm(make_float2(advected.x * 0.58f - t * 0.010f, advected.y * 0.72f), seed + 313ULL);
    float haze = saturate((broad * 0.48f + sheet * 0.36f + veil * 0.16f - 0.315f) * 1.85f);
    float depth = smooth(saturate((p.y + 0.60f) * 1.8f)) * smooth(saturate((0.54f - p.y) * 1.1f));
    float valley_hold = smooth(saturate((-p.y + 0.06f) * 2.0f));
    float plasma_cut = 1.0f - saturate(plasma * 0.32f);
    float wake = powf(saturate(plasma), 0.45f) * (0.08f + valley_hold * 0.08f);
    return (haze * depth * plasma_cut + wake * depth) * params.smoke_density;
}

__device__ float sample_scalar_field(const float* field, int w, int h, float2 uv) {
    float px = saturate(uv.x) * static_cast<float>(w - 1);
    float py = saturate(uv.y) * static_cast<float>(h - 1);
    int x0 = static_cast<int>(floorf(px));
    int y0 = static_cast<int>(floorf(py));
    int x1 = min(w - 1, x0 + 1);
    int y1 = min(h - 1, y0 + 1);
    float tx = px - floorf(px);
    float ty = py - floorf(py);
    float a = field[y0 * w + x0];
    float b = field[y0 * w + x1];
    float c = field[y1 * w + x0];
    float d = field[y1 * w + x1];
    float ab = a + (b - a) * tx;
    float cd = c + (d - c) * tx;
    return ab + (cd - ab) * ty;
}

__device__ float2 sample_velocity_field(const float2* field, int w, int h, float2 uv) {
    float px = saturate(uv.x) * static_cast<float>(w - 1);
    float py = saturate(uv.y) * static_cast<float>(h - 1);
    int x0 = static_cast<int>(floorf(px));
    int y0 = static_cast<int>(floorf(py));
    int x1 = min(w - 1, x0 + 1);
    int y1 = min(h - 1, y0 + 1);
    float tx = px - floorf(px);
    float ty = py - floorf(py);
    float2 a = field[y0 * w + x0];
    float2 b = field[y0 * w + x1];
    float2 c = field[y1 * w + x0];
    float2 d = field[y1 * w + x1];
    float2 ab = make_float2(a.x + (b.x - a.x) * tx, a.y + (b.y - a.y) * tx);
    float2 cd = make_float2(c.x + (d.x - c.x) * tx, c.y + (d.y - c.y) * tx);
    return make_float2(ab.x + (cd.x - ab.x) * ty, ab.y + (cd.y - ab.y) * ty);
}

__global__ void build_fields_kernel(float2* velocity, float* density, float* temperature, DhoolRenderParams params) {
    uint32_t x = blockIdx.x * blockDim.x + threadIdx.x;
    uint32_t y = blockIdx.y * blockDim.y + threadIdx.y;
    if (x >= params.field_width || y >= params.field_height) {
        return;
    }
    float aspect = static_cast<float>(params.width) / static_cast<float>(params.height);
    float2 uv = make_float2((static_cast<float>(x) + 0.5f) / params.field_width, (static_cast<float>(y) + 0.5f) / params.field_height);
    float2 p = uv_to_p(uv, aspect);
    float t = params.time_seconds;
    uint64_t seed = params.seed;
    uint32_t index = y * params.field_width + x;
    float plasma = plasma_mask(p, t, seed, params);
    velocity[index] = field_flow(p, t, seed, params);
    density[index] = smoke_layer(p, t, seed, params, plasma);
    temperature[index] = plasma;
}

__global__ void render_hdr_kernel(
    float4* hdr,
    const float2* velocity,
    const float* density,
    const float* temperature,
    DhoolRenderParams params
) {
    uint32_t x = blockIdx.x * blockDim.x + threadIdx.x;
    uint32_t y = blockIdx.y * blockDim.y + threadIdx.y;
    if (x >= params.width || y >= params.height) {
        return;
    }

    float aspect = static_cast<float>(params.width) / static_cast<float>(params.height);
    float2 uv = make_float2((static_cast<float>(x) + 0.5f) / params.width, (static_cast<float>(y) + 0.5f) / params.height);
    float2 p = make_float2((uv.x - 0.5f) * aspect, 0.5f - uv.y);
    float t = params.time_seconds;
    uint64_t seed = params.seed;

    float plasma = sample_scalar_field(temperature, params.field_width, params.field_height, uv);
    float smoke = sample_scalar_field(density, params.field_width, params.field_height, uv);
    float2 flow = sample_velocity_field(velocity, params.field_width, params.field_height, uv);
    float terrain = terrain_mask(p, t, seed);
    float rim = terrain_rim(p, t, seed);

    float central_void = expf(-((p.x * p.x) * 3.8f + (p.y - 0.13f + flow.y * 0.018f) * (p.y - 0.13f + flow.y * 0.018f) * 8.5f));
    float edge_focus = 0.64f + 0.36f * smooth(saturate((fabsf(p.x + flow.x * 0.012f) - 0.08f) * 2.2f));
    float lower_depth = smooth(saturate((-p.y + 0.16f) * 1.8f));
    float hot = powf(saturate(plasma), 1.34f) * params.plasma_emission_strength * (0.15f + params.plasma_core_temperature * 0.22f);
    float3 hot_core = make_float3(params.plasma_color_core[0], params.plasma_color_core[1], params.plasma_color_core[2]);
    float3 hot_edge = make_float3(params.plasma_color_edge[0], params.plasma_color_edge[1], params.plasma_color_edge[2]);
    float3 cool_smoke = make_float3(params.smoke_color_cool[0], params.smoke_color_cool[1], params.smoke_color_cool[2]);
    float3 warm_smoke = make_float3(params.smoke_color_warm[0], params.smoke_color_warm[1], params.smoke_color_warm[2]);
    float3 smoke_color = mix3(cool_smoke, warm_smoke, saturate(plasma * params.smoke_scattering * 0.35f + lower_depth * 0.12f + 0.10f));
    float3 background = make_float3(0.0100f, 0.0090f, 0.0082f);

    float3 color = background;
    float broad_veil = (1.0f - terrain) * (0.055f + 0.045f * fbm(make_float2(p.x * 1.05f - t * 0.006f, p.y * 0.85f), seed + 419ULL));
    float bottom_clear = 1.0f - smooth(saturate((-p.y - 0.08f) * 2.4f));
    broad_veil *= smooth(saturate((0.56f - p.y) * 1.10f)) * bottom_clear * (1.0f - central_void * 0.30f);
    color = add3(color, mul3(mix3(cool_smoke, warm_smoke, 0.62f), broad_veil));
    float ambient_haze = (1.0f - terrain) * (0.030f + smoke * 0.070f);
    ambient_haze *= 1.0f - central_void * 0.42f;
    color = add3(color, mul3(mix3(cool_smoke, warm_smoke, 0.45f), ambient_haze));
    color = add3(color, mul3(smoke_color, smoke * (0.68f + params.smoke_scattering * 0.42f)));

    float smoke_shadow = 1.0f - saturate(smoke * params.smoke_absorption * 0.16f);
    float3 flame_color = mix3(hot_edge, hot_core, saturate(plasma * 1.65f));
    float glow = powf(saturate(plasma * 0.82f), 0.58f) * params.plasma_emission_strength * 0.180f;
    color = add3(color, mul3(hot_edge, glow * edge_focus * (0.76f + smoke * 0.12f)));
    color = add3(color, mul3(flame_color, hot * edge_focus * smoke_shadow));
    color = mul3(color, 1.0f - central_void * (0.48f + 0.12f * (1.0f - lower_depth)));

    float rock = terrain * powf(fbm(make_float2(p.x * 9.5f + t * 0.004f, p.y * 9.0f), seed + 401ULL), 2.0f);
    float crack = terrain * powf(saturate(ridge(fbm(make_float2(p.x * 22.0f, p.y * 16.0f - t * 0.003f), seed + 409ULL))), 4.0f);
    float terrain_cover = terrain * (0.94f + 0.05f * lower_depth);
    color = mix3(color, make_float3(0.0013f, 0.0014f, 0.0018f), terrain_cover);
    color = add3(color, mul3(make_float3(0.165f, 0.130f, 0.100f), rock * 0.82f));
    color = add3(color, mul3(make_float3(0.40f, 0.22f, 0.090f), crack * rim * 0.82f));
    color = add3(color, mul3(hot_edge, rim * 1.17f * (0.10f + plasma * 0.34f + smoke * 0.08f)));
    color = add3(color, mul3(hot_edge, plasma * smoke * params.smoke_absorption * 0.12f * 0.46f));
    float foreground_shadow = smooth(saturate((-p.y - 0.10f) * 2.2f)) * (1.0f - terrain) * (1.0f - saturate(plasma * 1.7f));
    color = mul3(color, 1.0f - foreground_shadow * 0.55f);

    uint32_t index = y * params.width + x;
    hdr[index] = make_float4(color.x, color.y, color.z, 1.0f);
}

__global__ void init_particles_kernel(
    Particle* particles,
    const float2* velocity,
    const float* temperature,
    DhoolRenderParams params
) {
    uint32_t id = blockIdx.x * blockDim.x + threadIdx.x;
    if (id >= params.particles_count) {
        return;
    }
    float aspect = static_cast<float>(params.width) / static_cast<float>(params.height);
    uint64_t base_seed = params.seed ^ (static_cast<uint64_t>(id) * 0x9e3779b97f4a7c15ULL);

    // Deterministic spawn position: blend of terrain-biased and uniform.
    // ~60% biased toward lower scene (dust rising from ground), ~40% uniform.
    float hx = hash_u64_to_unit(base_seed + 11ULL);
    float raw_hy = hash_u64_to_unit(base_seed + 17ULL);
    float bias_select = hash_u64_to_unit(base_seed + 41ULL);
    float hy;
    if (bias_select < 0.6f) {
        // Bias toward bottom: y in [0.55, 0.95] (below the midline, near terrain)
        hy = 0.55f + raw_hy * 0.40f;
    } else {
        hy = raw_hy;
    }

    float heat_trait = hash_u64_to_unit(base_seed + 23ULL);
    float phase = hash_u64_to_unit(base_seed + 29ULL);
    float age = fmodf(phase + params.time_seconds / fmaxf(params.particles_lifetime, 0.01f), 1.0f);
    float age_seconds = age * fmaxf(params.particles_lifetime, 0.01f);

    float2 spawn_uv = make_float2(hx, hy);
    float2 pos = uv_to_p(spawn_uv, aspect);

    // Initial velocity: small random perturbation (not from flow field)
    float v0x = (hash_u64_to_unit(base_seed + 31ULL) - 0.5f) * 0.06f;
    float v0y = (hash_u64_to_unit(base_seed + 37ULL) - 0.5f) * 0.04f;
    float2 vel = make_float2(v0x, v0y);

    // Physics integration: sub-step forward from spawn time
    // Use 8 integration steps for smooth trajectories
    int substep_count = 8;
    float dt = age_seconds / fmaxf(static_cast<float>(substep_count), 1.0f);
    float accumulated_heat = 0.0f;

    for (int step = 0; step < substep_count; ++step) {
        float2 uv = p_to_uv(pos, aspect);
        uv.x = fminf(1.0f, fmaxf(0.0f, uv.x));
        uv.y = fminf(1.0f, fmaxf(0.0f, uv.y));

        // Sample local flow velocity and temperature
        float2 local_flow = sample_velocity_field(velocity, params.field_width, params.field_height, uv);
        float local_temp = sample_scalar_field(temperature, params.field_width, params.field_height, uv);

        // Accumulate heat from passing through hot regions
        accumulated_heat += local_temp * dt * 0.5f;

        // Gravity: constant downward pull
        float fy_gravity = -params.particles_gravity;

        // Stokes drag: F_drag = -drag * (v_particle - v_air * flow_coupling)
        float2 v_air = mul2(local_flow, params.particles_flow_coupling);
        float2 v_rel = sub2(vel, v_air);
        float2 drag_force = mul2(v_rel, -params.particles_drag);

        // Thermal updraft: hot regions push particles upward
        float thermal_force = local_temp * params.particles_thermal_lift;

        // Brownian jitter: deterministic stochastic noise per particle per step
        uint64_t jitter_seed = base_seed ^ (static_cast<uint64_t>(step) * 7919ULL)
                              ^ (static_cast<uint64_t>(params.frame_index) * 104729ULL);
        float jx = (hash_u64_to_unit(jitter_seed + 1ULL) - 0.5f) * 2.0f;
        float jy = (hash_u64_to_unit(jitter_seed + 2ULL) - 0.5f) * 2.0f;
        float jitter_scale = params.particles_jitter * sqrtf(fmaxf(dt, 1.0e-6f));

        // Total acceleration
        float ax = drag_force.x + jx * jitter_scale;
        float ay = fy_gravity + drag_force.y + thermal_force + jy * jitter_scale;

        // Euler integration
        vel.x += ax * dt;
        vel.y += ay * dt;
        pos.x += vel.x * dt;
        pos.y += vel.y * dt;
    }

    // Lifetime fade curve: smooth fade in and fade out
    float life = smooth(saturate(age * 5.0f)) * smooth(saturate((1.0f - age) * 2.8f));

    // Particle heat blends intrinsic heat trait with accumulated environmental heat
    float final_heat = saturate(heat_trait * 0.5f + accumulated_heat * 0.5f);

    particles[id].pos = pos;
    particles[id].vel = vel;
    particles[id].age = age;
    particles[id].heat = final_heat;
    particles[id].size = params.particles_size_min + (params.particles_size_max - params.particles_size_min) * heat_trait;
    // Alpha based on lifetime only — not gated by plasma temperature
    particles[id].alpha = life * (0.24f + final_heat * 0.76f);
}

__global__ void render_particles_kernel(float4* hdr, const Particle* particles, DhoolRenderParams params) {
    uint32_t id = blockIdx.x * blockDim.x + threadIdx.x;
    if (id >= params.particles_count) {
        return;
    }
    Particle particle = particles[id];
    if (particle.alpha <= 0.001f) {
        return;
    }
    float aspect = static_cast<float>(params.width) / static_cast<float>(params.height);
    float2 uv = p_to_uv(particle.pos, aspect);
    if (uv.x < -0.02f || uv.x > 1.02f || uv.y < -0.02f || uv.y > 1.02f) {
        return;
    }
    float ridge_line = terrain_base_y(particle.pos.x);
    if (particle.pos.y < ridge_line + 0.012f) {
        return;
    }
    float px = uv.x * params.width;
    float py = uv.y * params.height;
    float depth = smooth(saturate((-particle.pos.y + 0.18f) * 1.6f));
    float radius_px = fmaxf(0.22f, particle.size * (0.70f + depth * 0.55f));
    int extent = static_cast<int>(ceilf(radius_px * 4.0f));
    float2 dir = normalize2(make_float2(particle.vel.x / aspect, -particle.vel.y));
    if (fabsf(dir.x) + fabsf(dir.y) < 0.001f) {
        dir = make_float2(1.0f, 0.0f);
    }
    float3 ember_color = mix3(
        make_float3(params.plasma_color_edge[0], params.plasma_color_edge[1], params.plasma_color_edge[2]),
        make_float3(params.plasma_color_core[0], params.plasma_color_core[1], params.plasma_color_core[2]),
        0.65f);
    float strength = params.particles_ember_brightness * particle.alpha * (0.070f + depth * 0.042f);
    int center_x = static_cast<int>(floorf(px));
    int center_y = static_cast<int>(floorf(py));
    for (int oy = -extent; oy <= extent; ++oy) {
        int y = center_y + oy;
        if (y < 0 || y >= static_cast<int>(params.height)) {
            continue;
        }
        for (int ox = -extent; ox <= extent; ++ox) {
            int x = center_x + ox;
            if (x < 0 || x >= static_cast<int>(params.width)) {
                continue;
            }
            float2 d = make_float2(static_cast<float>(ox) / params.width, static_cast<float>(oy) / params.height);
            float along = dot2(d, dir);
            float cross = dot2(d, make_float2(-dir.y, dir.x));
            float radius_uv = radius_px / fminf(static_cast<float>(params.width), static_cast<float>(params.height));
            float intensity = expf(-(cross * cross) / (radius_uv * radius_uv * 1.2f) - (along * along) / (radius_uv * radius_uv * 6.0f));
            intensity *= strength;
            uint32_t index = static_cast<uint32_t>(y) * params.width + static_cast<uint32_t>(x);
            atomicAdd(&hdr[index].x, ember_color.x * intensity);
            atomicAdd(&hdr[index].y, ember_color.y * intensity);
            atomicAdd(&hdr[index].z, ember_color.z * intensity);
        }
    }
}

__global__ void threshold_downsample_kernel(
    const float4* src,
    float4* dst,
    int src_w,
    int src_h,
    int dst_w,
    int dst_h,
    DhoolRenderParams params
) {
    int x = blockIdx.x * blockDim.x + threadIdx.x;
    int y = blockIdx.y * blockDim.y + threadIdx.y;
    if (x >= dst_w || y >= dst_h) {
        return;
    }
    float3 sum = make_float3(0.0f, 0.0f, 0.0f);
    for (int oy = 0; oy < 2; ++oy) {
        for (int ox = 0; ox < 2; ++ox) {
            int sx = min(src_w - 1, x * 2 + ox);
            int sy = min(src_h - 1, y * 2 + oy);
            float4 c = src[sy * src_w + sx];
            float lum = c.x * 0.2126f + c.y * 0.7152f + c.z * 0.0722f;
            float k = saturate((lum - params.bloom_threshold) / fmaxf(lum, 1.0e-4f));
            sum.x += c.x * k;
            sum.y += c.y * k;
            sum.z += c.z * k;
        }
    }
    dst[y * dst_w + x] = make_float4(sum.x * 0.25f, sum.y * 0.25f, sum.z * 0.25f, 1.0f);
}

__global__ void downsample_kernel(const float4* src, float4* dst, int src_w, int src_h, int dst_w, int dst_h) {
    int x = blockIdx.x * blockDim.x + threadIdx.x;
    int y = blockIdx.y * blockDim.y + threadIdx.y;
    if (x >= dst_w || y >= dst_h) {
        return;
    }
    float3 sum = make_float3(0.0f, 0.0f, 0.0f);
    for (int oy = 0; oy < 2; ++oy) {
        for (int ox = 0; ox < 2; ++ox) {
            int sx = min(src_w - 1, x * 2 + ox);
            int sy = min(src_h - 1, y * 2 + oy);
            float4 c = src[sy * src_w + sx];
            sum.x += c.x;
            sum.y += c.y;
            sum.z += c.z;
        }
    }
    dst[y * dst_w + x] = make_float4(sum.x * 0.25f, sum.y * 0.25f, sum.z * 0.25f, 1.0f);
}

__device__ float3 sample_bloom_level(const float4* src, int w, int h, float2 uv) {
    float px = saturate(uv.x) * static_cast<float>(w - 1);
    float py = saturate(uv.y) * static_cast<float>(h - 1);
    int x0 = static_cast<int>(floorf(px));
    int y0 = static_cast<int>(floorf(py));
    int x1 = min(w - 1, x0 + 1);
    int y1 = min(h - 1, y0 + 1);
    float tx = px - floorf(px);
    float ty = py - floorf(py);
    float4 a = src[y0 * w + x0];
    float4 b = src[y0 * w + x1];
    float4 c = src[y1 * w + x0];
    float4 d = src[y1 * w + x1];
    float3 ab = make_float3(a.x + (b.x - a.x) * tx, a.y + (b.y - a.y) * tx, a.z + (b.z - a.z) * tx);
    float3 cd = make_float3(c.x + (d.x - c.x) * tx, c.y + (d.y - c.y) * tx, c.z + (d.z - c.z) * tx);
    return make_float3(ab.x + (cd.x - ab.x) * ty, ab.y + (cd.y - ab.y) * ty, ab.z + (cd.z - ab.z) * ty);
}

__device__ float3 aces_tonemap(float3 x) {
    const float a = 2.51f;
    const float b = 0.03f;
    const float c = 2.43f;
    const float d = 0.59f;
    const float e = 0.14f;
    return make_float3(
        saturate((x.x * (a * x.x + b)) / (x.x * (c * x.x + d) + e)),
        saturate((x.y * (a * x.y + b)) / (x.y * (c * x.y + d) + e)),
        saturate((x.z * (a * x.z + b)) / (x.z * (c * x.z + d) + e))
    );
}

__global__ void final_grade_kernel(const float4* hdr, uchar4* out, BloomView bloom, DhoolRenderParams params) {
    uint32_t x = blockIdx.x * blockDim.x + threadIdx.x;
    uint32_t y = blockIdx.y * blockDim.y + threadIdx.y;
    if (x >= params.width || y >= params.height) {
        return;
    }
    uint32_t index = y * params.width + x;
    float2 uv = make_float2((static_cast<float>(x) + 0.5f) / params.width, (static_cast<float>(y) + 0.5f) / params.height);
    float4 base4 = hdr[index];
    float3 color = make_float3(base4.x, base4.y, base4.z);
    float radius_scale = 0.0015f + params.bloom_radius * 0.0025f;
    float3 bloom_sum = make_float3(0.0f, 0.0f, 0.0f);
    float weight_sum = 0.0f;
    for (int i = 0; i < bloom.level_count; ++i) {
        float spread = static_cast<float>(i + 1) * radius_scale;
        float weight = powf(0.74f, static_cast<float>(i));
        float3 center = sample_bloom_level(bloom.levels[i], bloom.widths[i], bloom.heights[i], uv);
        float3 east = sample_bloom_level(bloom.levels[i], bloom.widths[i], bloom.heights[i], make_float2(uv.x + spread, uv.y));
        float3 west = sample_bloom_level(bloom.levels[i], bloom.widths[i], bloom.heights[i], make_float2(uv.x - spread, uv.y));
        float3 north = sample_bloom_level(bloom.levels[i], bloom.widths[i], bloom.heights[i], make_float2(uv.x, uv.y - spread));
        float3 south = sample_bloom_level(bloom.levels[i], bloom.widths[i], bloom.heights[i], make_float2(uv.x, uv.y + spread));
        float3 level = mul3(add3(add3(add3(add3(center, east), west), north), south), 0.2f);
        bloom_sum = add3(bloom_sum, mul3(level, weight));
        weight_sum += weight;
    }
    if (weight_sum > 0.0f) {
        color = add3(color, mul3(bloom_sum, params.bloom_strength / weight_sum));
    }

    color = mul3(color, exp2f(params.grade_exposure));
    color = aces_tonemap(color);
    color.x = saturate((color.x - params.grade_black_point) / fmaxf(1.0f - params.grade_black_point, 1.0e-4f));
    color.y = saturate((color.y - params.grade_black_point) / fmaxf(1.0f - params.grade_black_point, 1.0e-4f));
    color.z = saturate((color.z - params.grade_black_point) / fmaxf(1.0f - params.grade_black_point, 1.0e-4f));
    color = make_float3(
        saturate((color.x - 0.5f) * params.grade_contrast + 0.5f),
        saturate((color.y - 0.5f) * params.grade_contrast + 0.5f),
        saturate((color.z - 0.5f) * params.grade_contrast + 0.5f)
    );
    float lum = color.x * 0.2126f + color.y * 0.7152f + color.z * 0.0722f;
    color = make_float3(
        saturate(lum + (color.x - lum) * params.grade_saturation),
        saturate(lum + (color.y - lum) * params.grade_saturation),
        saturate(lum + (color.z - lum) * params.grade_saturation)
    );
    float2 centered = make_float2(uv.x - 0.5f, uv.y - 0.5f);
    float vignette = 1.0f - params.grade_vignette * saturate(dot2(centered, centered) * 1.65f);
    color = mul3(color, vignette);
    float grain = (hash_u64_to_unit(params.seed ^ (static_cast<uint64_t>(x) << 32) ^ static_cast<uint64_t>(y) ^ static_cast<uint64_t>(params.frame_index) * 0x9e37ULL) - 0.5f) * params.grade_grain;
    color = make_float3(saturate(color.x + grain), saturate(color.y + grain), saturate(color.z + grain));

    out[index] = make_uchar4(
        static_cast<unsigned char>(saturate(color.x) * 255.0f + 0.5f),
        static_cast<unsigned char>(saturate(color.y) * 255.0f + 0.5f),
        static_cast<unsigned char>(saturate(color.z) * 255.0f + 0.5f),
        255
    );
}

void clear_device_info(DhoolCudaDeviceInfo* out) {
    std::memset(out, 0, sizeof(DhoolCudaDeviceInfo));
}

void clear_report(DhoolRenderReport* report) {
    std::memset(report, 0, sizeof(DhoolRenderReport));
}

void set_error(DhoolRenderReport* report, cudaError_t error, const char* context) {
    report->status = 0;
    report->error_code = static_cast<int>(error);
    std::snprintf(report->error_message, sizeof(report->error_message), "%s: %s", context, cudaGetErrorString(error));
}

bool ensure_device_buffer(DeviceBuffer& buffer, size_t bytes, DhoolRenderReport* report, const char* context) {
    if (bytes == 0) {
        return true;
    }
    if (buffer.ptr != nullptr && buffer.capacity >= bytes) {
        return true;
    }

    void* next = nullptr;
    cudaError_t err = cudaMalloc(&next, bytes);
    if (err != cudaSuccess) {
        set_error(report, err, context);
        return false;
    }
    if (buffer.ptr != nullptr) {
        cudaFree(buffer.ptr);
    }
    buffer.ptr = next;
    buffer.capacity = bytes;
    return true;
}

bool ensure_host_buffer(HostBuffer& buffer, size_t bytes, DhoolRenderReport* report, const char* context) {
    if (bytes == 0) {
        return true;
    }
    if (buffer.ptr != nullptr && buffer.capacity >= bytes) {
        return true;
    }

    void* next = nullptr;
    cudaError_t err = cudaHostAlloc(&next, bytes, cudaHostAllocPortable);
    if (err != cudaSuccess) {
        set_error(report, err, context);
        return false;
    }
    if (buffer.ptr != nullptr) {
        cudaFreeHost(buffer.ptr);
    }
    buffer.ptr = next;
    buffer.capacity = bytes;
    return true;
}

bool ensure_events(RenderCache& cache, DhoolRenderReport* report) {
    if (cache.events_ready) {
        return true;
    }
    cudaError_t err = cudaEventCreate(&cache.gpu_start);
    if (err != cudaSuccess) {
        set_error(report, err, "cudaEventCreate gpu_start");
        return false;
    }
    err = cudaEventCreate(&cache.gpu_stop);
    if (err != cudaSuccess) {
        set_error(report, err, "cudaEventCreate gpu_stop");
        return false;
    }
    err = cudaEventCreate(&cache.copy_start);
    if (err != cudaSuccess) {
        set_error(report, err, "cudaEventCreate copy_start");
        return false;
    }
    err = cudaEventCreate(&cache.copy_stop);
    if (err != cudaSuccess) {
        set_error(report, err, "cudaEventCreate copy_stop");
        return false;
    }
    cache.events_ready = true;
    return true;
}

extern "C" int dhool_cuda_device_info(DhoolCudaDeviceInfo* out) {
    if (out == nullptr) {
        return 0;
    }
    clear_device_info(out);
    int count = 0;
    cudaError_t err = cudaGetDeviceCount(&count);
    if (err != cudaSuccess || count <= 0) {
        out->available = 0;
        out->error_code = static_cast<int>(err);
        std::snprintf(out->error_message, sizeof(out->error_message), "cudaGetDeviceCount failed: %s", cudaGetErrorString(err));
        return 0;
    }

    cudaDeviceProp prop {};
    err = cudaGetDeviceProperties(&prop, 0);
    if (err != cudaSuccess) {
        out->available = 0;
        out->error_code = static_cast<int>(err);
        std::snprintf(out->error_message, sizeof(out->error_message), "cudaGetDeviceProperties failed: %s", cudaGetErrorString(err));
        return 0;
    }

    int driver = 0;
    int runtime = 0;
    cudaDriverGetVersion(&driver);
    cudaRuntimeGetVersion(&runtime);
    out->available = 1;
    std::snprintf(out->name, sizeof(out->name), "%s", prop.name);
    out->driver_version = driver;
    out->runtime_version = runtime;
    out->global_memory_bytes = static_cast<unsigned long long>(prop.totalGlobalMem);
    out->multiprocessor_count = prop.multiProcessorCount;
    return 1;
}

extern "C" int dhool_cuda_render_rgba8(
    const DhoolRenderParams* params,
    uint8_t* host_rgba,
    DhoolRenderReport* report
) {
    if (params == nullptr || host_rgba == nullptr || report == nullptr) {
        return 0;
    }
    clear_report(report);
    cudaError_t err = cudaSetDevice(0);
    if (err != cudaSuccess) {
        set_error(report, err, "cudaSetDevice");
        return 0;
    }

    const size_t pixels = static_cast<size_t>(params->width) * static_cast<size_t>(params->height);
    const size_t field_pixels = static_cast<size_t>(params->field_width) * static_cast<size_t>(params->field_height);
    const size_t rgba_bytes = pixels * sizeof(uchar4);
    std::lock_guard<std::mutex> lock(g_render_cache_mutex);
    RenderCache& cache = g_render_cache;
    if (!ensure_events(cache, report)
        || !ensure_device_buffer(cache.hdr, pixels * sizeof(float4), report, "cudaMalloc hdr")
        || !ensure_device_buffer(cache.ldr, rgba_bytes, report, "cudaMalloc ldr")
        || !ensure_device_buffer(cache.velocity, field_pixels * sizeof(float2), report, "cudaMalloc velocity field")
        || !ensure_device_buffer(cache.density, field_pixels * sizeof(float), report, "cudaMalloc density field")
        || !ensure_device_buffer(cache.temperature, field_pixels * sizeof(float), report, "cudaMalloc temperature field")
        || !ensure_device_buffer(cache.particles, static_cast<size_t>(params->particles_count) * sizeof(Particle), report, "cudaMalloc particle buffer")
        || !ensure_host_buffer(cache.readback, rgba_bytes, report, "cudaHostAlloc readback buffer")) {
        return 0;
    }

    float4* hdr = static_cast<float4*>(cache.hdr.ptr);
    uchar4* ldr = static_cast<uchar4*>(cache.ldr.ptr);
    float2* velocity = static_cast<float2*>(cache.velocity.ptr);
    float* density = static_cast<float*>(cache.density.ptr);
    float* temperature = static_cast<float*>(cache.temperature.ptr);
    Particle* particles = static_cast<Particle*>(cache.particles.ptr);
    float4* bloom_levels[8] {};
    int bloom_w[8] {};
    int bloom_h[8] {};
    BloomView bloom {};
    int level_count = std::max(1, std::min(8, static_cast<int>(params->bloom_pyramid_levels)));

    int previous_w = static_cast<int>(params->width);
    int previous_h = static_cast<int>(params->height);
    for (int i = 0; i < level_count; ++i) {
        bloom_w[i] = std::max(1, previous_w / 2);
        bloom_h[i] = std::max(1, previous_h / 2);
        size_t bloom_bytes = static_cast<size_t>(bloom_w[i]) * static_cast<size_t>(bloom_h[i]) * sizeof(float4);
        if (!ensure_device_buffer(cache.bloom_levels[i], bloom_bytes, report, "cudaMalloc bloom level")) {
            return 0;
        }
        bloom_levels[i] = static_cast<float4*>(cache.bloom_levels[i].ptr);
        previous_w = bloom_w[i];
        previous_h = bloom_h[i];
    }

    dim3 block(16, 16);
    dim3 full_grid((params->width + block.x - 1) / block.x, (params->height + block.y - 1) / block.y);
    dim3 field_grid((params->field_width + block.x - 1) / block.x, (params->field_height + block.y - 1) / block.y);
    cudaEventRecord(cache.gpu_start);
    build_fields_kernel<<<field_grid, block>>>(velocity, density, temperature, *params);
    err = cudaGetLastError();
    if (err != cudaSuccess) {
        set_error(report, err, "build_fields_kernel");
        goto cleanup;
    }
    render_hdr_kernel<<<full_grid, block>>>(hdr, velocity, density, temperature, *params);
    err = cudaGetLastError();
    if (err != cudaSuccess) {
        set_error(report, err, "render_hdr_kernel");
        goto cleanup;
    }
    if (params->particles_count > 0) {
        dim3 particle_block(256);
        dim3 particle_grid((params->particles_count + particle_block.x - 1) / particle_block.x);
        init_particles_kernel<<<particle_grid, particle_block>>>(particles, velocity, temperature, *params);
        err = cudaGetLastError();
        if (err != cudaSuccess) {
            set_error(report, err, "init_particles_kernel");
            goto cleanup;
        }
        render_particles_kernel<<<particle_grid, particle_block>>>(hdr, particles, *params);
        err = cudaGetLastError();
        if (err != cudaSuccess) {
            set_error(report, err, "render_particles_kernel");
            goto cleanup;
        }
    }

    for (int i = 0; i < level_count; ++i) {
        dim3 grid((bloom_w[i] + block.x - 1) / block.x, (bloom_h[i] + block.y - 1) / block.y);
        if (i == 0) {
            threshold_downsample_kernel<<<grid, block>>>(hdr, bloom_levels[i], params->width, params->height, bloom_w[i], bloom_h[i], *params);
        } else {
            downsample_kernel<<<grid, block>>>(bloom_levels[i - 1], bloom_levels[i], bloom_w[i - 1], bloom_h[i - 1], bloom_w[i], bloom_h[i]);
        }
        err = cudaGetLastError();
        if (err != cudaSuccess) {
            set_error(report, err, "bloom downsample kernel");
            goto cleanup;
        }
    }

    bloom.level_count = level_count;
    for (int i = 0; i < level_count; ++i) {
        bloom.levels[i] = bloom_levels[i];
        bloom.widths[i] = bloom_w[i];
        bloom.heights[i] = bloom_h[i];
    }
    final_grade_kernel<<<full_grid, block>>>(hdr, ldr, bloom, *params);
    err = cudaGetLastError();
    if (err != cudaSuccess) {
        set_error(report, err, "final_grade_kernel");
        goto cleanup;
    }
    cudaEventRecord(cache.gpu_stop);
    err = cudaEventSynchronize(cache.gpu_stop);
    if (err != cudaSuccess) {
        set_error(report, err, "cudaEventSynchronize gpu_stop");
        goto cleanup;
    }
    cudaEventElapsedTime(&report->gpu_ms, cache.gpu_start, cache.gpu_stop);

    cudaEventRecord(cache.copy_start);
    err = cudaMemcpy(cache.readback.ptr, ldr, rgba_bytes, cudaMemcpyDeviceToHost);
    if (err != cudaSuccess) {
        set_error(report, err, "cudaMemcpy readback");
        goto cleanup;
    }
    cudaEventRecord(cache.copy_stop);
    err = cudaEventSynchronize(cache.copy_stop);
    if (err != cudaSuccess) {
        set_error(report, err, "cudaEventSynchronize copy_stop");
        goto cleanup;
    }
    cudaEventElapsedTime(&report->readback_ms, cache.copy_start, cache.copy_stop);
    std::memcpy(host_rgba, cache.readback.ptr, rgba_bytes);
    report->status = 1;

cleanup:
    return report->status;
}
