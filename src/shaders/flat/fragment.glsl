#version 330

in vec3 v_normal;
in vec3 v_position;
in float v_value;

out vec4 frag_color;

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
    // calculate color;
    vec3 v_color = jet_color(v_value);

    // calculate fragment color
    frag_color = pow(vec4(v_color, 1.0), vec4(2.2, 2.2, 2.2, 1.0));
}
