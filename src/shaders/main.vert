#version 330

in vec3 position;
in vec3 normal;
in float value;

out vec3 v_normal;
out vec3 v_position;
out vec3 v_color;
out vec4 shadow_position;

uniform mat4 perspective;
uniform mat4 view;
uniform mat4 model;

uniform mat4 shadow_perspective;
uniform mat4 shadow_view;
const mat4 shadow_bias = mat4(
    vec4(0.5, 0.0, 0.0, 0.0),
    vec4(0.0, 0.5, 0.0, 0.0),
    vec4(0.0, 0.0, 0.5, 0.0),
    vec4(0.5, 0.5, 0.5, 1.0));

vec3 jet_color(float value) {
    // correct value space
    float normalized_value = 0.5 * value + 0.5;

    // calculate colors
    float red = min(max(-4.0 * abs(normalized_value - 0.75) + 1.5, 0.0), 1.0);
    float green = min(max(-4.0 * abs(normalized_value - 0.5) + 1.5, 0.0), 1.0);
    float blue = min(max(-4.0 * abs(normalized_value - 0.25) + 1.5, 0.0), 1.0);

    return vec3(red, green, blue);
}

void main() {
    // calculate color
    v_color = jet_color(value);

    // position of vertex in clip space
    gl_Position = perspective * view * model * vec4(position, 1.0);

    // normal of vertex in camera space
    v_normal = transpose(inverse(mat3(view * model))) * normal;

    // position of vertex in camera space
    v_position = (view * model * vec4(position, 1.0)).xyz;

    // position of vertex in shadow space
    shadow_position = shadow_bias * shadow_perspective * shadow_view * model * vec4(position, 1.0);
}
