#include "cloth.h"
#include <iostream>
#include <random>

Cloth::Cloth(int n, int m) : g(9.81f), m(0.01f), g_on(true) {
	for (int i = 0; i < n; i++) {
		std::vector<Point> row;
		for (int j = 0; j < m; j++) {
			row.emplace_back(j, i);
		}
		points.push_back(row);
	}

	points[n - 1][0].fixed = true;
	points[n - 1][0].static_point = true;
	points[n - 1][m - 1].fixed = true;
	points[n - 1][m - 1].static_point = true;

	for (int i = 0; i < n; i++) {
		for (int j = 0; j < m; j++) {
			if (i < n - 1) {
				springs.emplace_back(std::make_pair(i, j), std::make_pair(i + 1, j), 1.0f, 10.0f, 0.03f);
			}
			if (j < m - 1) {
				springs.emplace_back(std::make_pair(i, j), std::make_pair(i, j + 1), 1.0f, 10.0f, 0.03f);
			}
		}
	}
}

void Cloth::simulate(float dt) {
    std::vector<std::vector<std::pair<float, float>>> forces(points.size(), std::vector<std::pair<float, float>>(points[0].size(), { 0.0f, 0.0f }));

    for (int i = 0; i < points.size(); ++i) {
        for (int j = 0; j < points[i].size(); ++j) {
            float total_force_x = 0.0f;
            float total_force_y = 0.0f;

            for (const auto& spring : springs) {
                Point& point1 = points[spring.p1.first][spring.p1.second];
                Point& point2 = points[spring.p2.first][spring.p2.second];

                float dx = point2.x - point1.x;
                float dy = point2.y - point1.y;

                float dist = std::sqrt(dx * dx + dy * dy);
                float magnitude = spring.spring_coeff * (dist - spring.rest_length);

                float spring_force_x = (dist != 0.0f) ? (magnitude * dx) / dist : 0.0f;
                float spring_force_y = (dist != 0.0f) ? (magnitude * dy) / dist : 0.0f;

                float damping_force_x = -point1.vx * spring.damp_coeff;
                float damping_force_y = -point1.vy * spring.damp_coeff;

                if (point1.x == points[i][j].x && point1.y == points[i][j].y) {
                    total_force_x += spring_force_x + damping_force_x;
                    total_force_y += spring_force_y + damping_force_y;
                }
                else if (point2.x == points[i][j].x && point2.y == points[i][j].y) {
                    total_force_x -= spring_force_x - damping_force_x;
                    total_force_y -= spring_force_y - damping_force_y;
                }
            }

            float gravity_force_x = 0.0f;
            float gravity_force_y = g_on ? -g * m : 0.0f;

            std::random_device rd;
            std::mt19937 gen(rd());
            std::uniform_real_distribution<float> dis(-1.0f, 1.0f);

            float ext_force_x = dis(gen) * points[i][j].ext_m;
            float ext_force_y = dis(gen) * points[i][j].ext_m;

            total_force_x += gravity_force_x + ext_force_x;
            total_force_y += gravity_force_y + ext_force_y;

            forces[i][j] = { total_force_x, total_force_y };
        }
    }

    for (int i = 0; i < points.size(); ++i) {
        for (int j = 0; j < points[i].size(); ++j) {
            if (points[i][j].fixed) {
                continue;
            }

            float fx = forces[i][j].first;
            float fy = forces[i][j].second;

            points[i][j].ax = fx / m;
            points[i][j].ay = fy / m;

            float prev_x = points[i][j].x;
            float prev_y = points[i][j].y;

            points[i][j].x += points[i][j].vx * dt + 0.5f * points[i][j].ax * dt * dt;
            points[i][j].y += points[i][j].vy * dt + 0.5f * points[i][j].ay * dt * dt;

            if (points[i][j].y < -32.0f) {
                points[i][j].y = -32.0f;
                points[i][j].vy = 0.0f;
            }

            float new_vx = (points[i][j].x - prev_x) / dt;
            float new_vy = (points[i][j].y - prev_y) / dt;

            if (points[i][j].y == -32.0f) {
                points[i][j].vx = -new_vy;
                points[i][j].vy = -new_vy;
            }
            else {
                points[i][j].vx = new_vx;
                points[i][j].vy = new_vy;
            }
        }
    }
}
