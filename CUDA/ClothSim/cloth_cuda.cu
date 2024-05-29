#include <cuda_runtime.h>
#include "cloth.h"
#include "device_launch_parameters.h"
#include <curand_kernel.h>
#include <time.h>

__global__ void simulatePoint(Point* points, Spring* springs, int N, int M, int num_springs, float dt, float g, float g_on, float m) {
	int idx = blockIdx.x * blockDim.x + threadIdx.x;
	int num_points = N * M;
	if (idx >= num_points) return;

	Point* p = &points[idx];

	if (p->fixed) return;

	float fx = 0.0f, fy = 0.0f;

	// calculate spring forces and damping
	for (int i = 0; i < num_springs; i++)
	{
		Spring& s = springs[i];

		int row1 = s.p1.first;
		int col1 = s.p1.second;

		int row2 = s.p2.first;
		int col2 = s.p2.second;

		// Get pointers to the connected points
		Point* p1 = &points[row1 * M + col1];
		Point* p2 = &points[row2 * M + col2];

		// Apply forces only if p is one of the points connected by the spring
		if (p == p1 || p == p2) {
			float dx = p2->x - p1->x;
			float dy = p2->y - p1->y;

			float dist = sqrtf(dx * dx + dy * dy);
			float magnitude = s.spring_coeff * (dist - s.rest_length);

			float spring_force_x = (dist != 0.0f) ? (magnitude * dx / dist) : 0.0f;
			float spring_force_y = (dist != 0.0f) ? (magnitude * dy / dist) : 0.0f;

			float damping_force_x = -p1->vx * s.damp_coeff;
			float damping_force_y = -p1->vy * s.damp_coeff;

			if (p == p1) {
				fx += spring_force_x + damping_force_x;
				fy += spring_force_y + damping_force_y;
			}
			else {
				fx -= spring_force_x - damping_force_x;
				fy -= spring_force_y - damping_force_y;
			}
		}
	}

	float gravity_force_y = g_on ? -g * m : 0.0f;

	fy += gravity_force_y;

	// random external force
	curandState state;
	curand_init(clock() * idx, 0, 0, &state);

	float ext_force_x = curand_uniform(&state) * 2.0f - 1.0f;
	float ext_force_y = curand_uniform(&state) * 2.0f - 1.0f;

	ext_force_x *= p->ext_m;
	ext_force_y *= p->ext_m;

	fx += ext_force_x;
	fy += ext_force_y;

	p->ax = fx / m;
	p->ay = fy / m;

	float prev_x = p->x;
	float prev_y = p->y;

	p->x += p->vx * dt + 0.5f * p->ax * dt * dt;
	p->y += p->vy * dt + 0.5f * p->ay * dt * dt;

	//floor collision
	if (p->y < -16.0f) {
		p->y = -16.0f;
		p->vy = 0.0f;
	}

	float new_vx = (p->x - prev_x) / dt;
	float new_vy = (p->y - prev_y) / dt;

	if (p->y == -16.0f) {
		p->vx = -new_vy;
		p->vy = -new_vy;
	}
	else {
		p->vx = new_vx;
		p->vy = new_vy;
	}
}

extern "C" void simulateKernel(Point * points, Spring * springs, int N, int M, int num_springs, float dt, float g, bool g_on, float m) {
	//printf("CUDA simulation started...\n");
	int num_points = N * M;

	Point* d_points;
	Spring* d_springs;

	cudaError_t cudaStatus;

	cudaStatus = cudaMalloc(&d_points, num_points * sizeof(Point));
	if (cudaStatus != cudaSuccess) {
		fprintf(stderr, "cudaMalloc failed!");
	}

	cudaStatus = cudaMalloc(&d_springs, num_springs * sizeof(Spring));
	if (cudaStatus != cudaSuccess) {
		fprintf(stderr, "cudaMalloc failed!");
	}
	

	cudaStatus = cudaMemcpy(d_points, points, num_points * sizeof(Point), cudaMemcpyHostToDevice);
	if (cudaStatus != cudaSuccess) {
		fprintf(stderr, "cudaMemcpy failed!");
	}

	cudaStatus = cudaMemcpy(d_springs, springs, num_springs * sizeof(Spring), cudaMemcpyHostToDevice);
	if (cudaStatus != cudaSuccess) {
		fprintf(stderr, "cudaMemcpy failed!");
	}

	int blockSize = 512;

	int numBlocks = (num_points + blockSize - 1) / blockSize;

	simulatePoint << <numBlocks, blockSize >> > (d_points, d_springs, N, M, num_springs, dt, g, g_on, m);

	cudaDeviceSynchronize();
	cudaGetLastError();

	cudaMemcpy(points, d_points, num_points * sizeof(Point), cudaMemcpyDeviceToHost);

	cudaFree(d_points);
	cudaFree(d_springs);
}
