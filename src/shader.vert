#version 450
precision highp float;
precision highp int;
layout(location = 0) out vec3 v_color;


layout(std140) uniform CB{
	uint point_count;
	float line_thickness;

	float radius0;
 	float initial_phase0;
 	float period0;
 	float initial_amplitude0;
 	float amplitude_decay0;

	float radius1;
 	float initial_phase1;
 	float period1;
 	float initial_amplitude1;
 	float amplitude_decay1;
};


#define PI 3.1415926535897932384626433832795028841971693993751058209749445923078164062

vec2 pendulum(float radius,
 			  float initial_phase,
 			  float period,
 			  float initial_amplitude,
 			  float amplitude_decay,
 			  float t) {
 	float tp = t;

	// The pendulum moves as a sine wave within
	float base_movement = sin(initial_phase + tp * period);

	float amplitude = initial_amplitude * pow(amplitude_decay, t);
	
	float current_angle = PI * 1.5 + amplitude * base_movement;

	return radius * vec2(
		cos(current_angle),
		sin(current_angle));
}

vec2 pointPos(uint index) {
	float t = index * 0.0001;
	vec2 p0 = pendulum(
			radius0,
		 	initial_phase0,
			period0,
		 	initial_amplitude0,
			amplitude_decay0,
			t);

	vec2 p1 = pendulum(
			radius1,
		 	initial_phase1,
			period1,
		 	initial_amplitude1,
			amplitude_decay1,
			t);

	return p0 + p1;
		
}

void main() {

    uint rectIndex = gl_VertexID / 2;
    uint localIndex = gl_VertexID % 2;

    /*
         *      0--1 3
         *      | / /|
         *      2/ 4-5
         */
    uint localCenterIndex = rectIndex + (localIndex & 1);
    float lateralOffset = line_thickness;
    if (localIndex == 1){
        lateralOffset *= -1;
    }

    vec2 localPos = pointPos(localCenterIndex);
    vec2 nextPos = pointPos(localCenterIndex+1);
    vec2 tangent = normalize(nextPos - localPos);
    vec2 perpDir = vec2(-tangent.y, tangent.x);

    vec2 vertPos = localPos + perpDir * lateralOffset;

	float color_t = gl_VertexID / float(point_count);

    v_color = vec3(
    		0.5 + 0.5 * sin(color_t * 2.0*PI),
    		0.5 + 0.5 * cos(color_t * 2.0*PI),
    		1.0);

    gl_Position = vec4(vertPos, 0.0, 1.0);
}
