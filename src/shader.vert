#version 300 es
precision highp float;
precision highp int;
out vec3 v_color;


layout(std140) uniform CB{
	float ar;
	float point_count;
	float zoom;
	float line_thickness;

	float radius0;
 	float initial_phase0;
 	float cycle_count0;
 	float fractional_cycles0;
 	float initial_amplitude0;
 	float amplitude_decay0;
 	float rotation0;

	float radius1;
 	float initial_phase1;
 	float cycle_count1;
 	float fractional_cycles1;
 	float initial_amplitude1;
 	float amplitude_decay1;
 	float rotation1;
};


#define PI 3.1415926535897932384626433832795028841971693993751058209749445923078164062

vec2 pendulum(float radius,
 			  float initial_phase,
 			  float cycle_count,
 			  float fractional_cycles,
 			  float initial_amplitude,
 			  float amplitude_decay,
 			  float rotation,
 			  float t) {
 	float tp = t;

	// The pendulum moves as a sine wave within
	float base_movement = sin(initial_phase + tp * 2.0 * PI * (cycle_count + fractional_cycles));

	float amplitude = initial_amplitude * pow(amplitude_decay, t);
	
	float current_angle = rotation + amplitude * base_movement;

	return radius * vec2(
		cos(current_angle),
		sin(current_angle));
}

vec2 pointPos(int index) {
	float t = float(index) * 0.0001;
	vec2 p0 = pendulum(
			radius0,
		 	initial_phase0,
			cycle_count0,
			fractional_cycles0,
		 	initial_amplitude0,
			amplitude_decay0,
			rotation0,
			t);

	vec2 p1 = pendulum(
			radius1,
		 	initial_phase1,
			cycle_count1,
			fractional_cycles1,
		 	initial_amplitude1,
			amplitude_decay1,
			rotation1,
			t);

	return p0 + p1;
		
}

void main() {

    int rectIndex = gl_VertexID / 2;
    int localIndex = gl_VertexID % 2;

    /*
         *      0--1 3
         *      | / /|
         *      2/ 4-5
         */
    int localCenterIndex = rectIndex + (localIndex & 1);
    float lateralOffset = line_thickness;
    if (localIndex == 1){
        lateralOffset *= -1.0;
    }

    vec2 localPos = pointPos(localCenterIndex);
    vec2 nextPos = pointPos(localCenterIndex+1);
    vec2 tangent = normalize(nextPos - localPos);
    vec2 perpDir = vec2(-tangent.y, tangent.x);

    vec2 vertPos = zoom * (localPos + perpDir * lateralOffset) * vec2(1.0/ar, 1.0);

	float color_t = float(gl_VertexID) / float(point_count);

    v_color = vec3(
    		0.5 + 0.5 * sin(color_t * 2.0*PI),
    		0.5 + 0.5 * cos(color_t * 2.0*PI),
    		1.0);

    gl_Position = vec4(vertPos, 0.0, 1.0);
}
