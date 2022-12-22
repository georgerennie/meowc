// A sketch of an unverified SAT proof checker, used for prototyping ideas for
// the meowc-check sat checker

#include <cassert>
#include <chrono>
#include <cstdint>
#include <fstream>
#include <iostream>
#include <iterator>
#include <sstream>
#include <stdexcept>
#include <string>
#include <tuple>
#include <vector>

enum class TriBool : uint8_t {
	None  = 0x00,
	False = 0x01,
	True  = 0x02,
};

using Var           = uint32_t;
using AssignmentVec = std::vector<TriBool>;

class Lit {
public:
	constexpr Lit(const Var var, const bool is_pos) : lit(var | (is_pos ? pos_mask : 0)) {
		assert(var != 0);
	}

	constexpr Var     var() const { return lit & ~pos_mask; }
	constexpr bool    is_pos() const { return lit & pos_mask; }
	constexpr TriBool tri_bool() const { return is_pos() ? TriBool::True : TriBool::False; }

	bool sat_by(const AssignmentVec& assignment) const { return assignment[var()] == tri_bool(); }

	bool operator==(const Lit& rhs) const { return lit == rhs.lit; }

	friend std::ostream& operator<<(std::ostream& out, const Lit& lit) {
		if (!lit.is_pos())
			out << '-';
		return out << lit.var();
	}

	static Lit make_lit(const std::string& s) {
		bool      is_pos = (s[0] != '-');
		const Var val    = std::stoi(is_pos ? s : s.substr(1));
		return Lit{val, is_pos};
	}

private:
	static constexpr Var pos_mask = 0x80000000;
	Var                  lit;
};

using Clause     = std::vector<Lit>;
using Formula    = std::vector<Clause>;
using Assignment = std::vector<Lit>;

AssignmentVec to_assignment_vec(const Assignment& assignment, const std::size_t num_vars) {
	AssignmentVec assignment_vec{num_vars + 1, TriBool::None};

	for (const auto lit : assignment) {
		const auto var = lit.var();
		assert(var <= num_vars);
		auto& value = assignment_vec[var];

		if (value == TriBool::None) {
			value = lit.tri_bool();
			continue;
		}

		assert(value == lit.tri_bool());
	}

	return assignment_vec;
}

bool is_sat(const Formula& formula, const AssignmentVec& assignment) {
	for (const auto& clause : formula) {
		bool clause_sat = false;
		for (const auto lit : clause) {
			if (lit.sat_by(assignment)) {
				clause_sat = true;
				break;
			}
		}

		if (!clause_sat)
			return false;
	}

	return true;
}

bool check_sat(const Formula& formula, const Assignment& assignment, const std::size_t num_vars) {
	return is_sat(formula, to_assignment_vec(assignment, num_vars));
}

std::vector<std::string> split(std::string const& input) {
	std::istringstream       buffer(input);
	std::vector<std::string> ret{std::istream_iterator<std::string>(buffer), {}};
	return ret;
}

std::tuple<Formula, std::size_t> parse_formula(std::ifstream& fs) {
	Formula     formula;
	std::size_t num_variables = 0;
	std::size_t num_clauses   = 0;

	std::string line;
	// Read comment/problem statement
	while (std::getline(fs, line)) {
		if (line.size() == 0 || line[0] == 'c')
			continue;

		if (line[0] == 'p') {
			const auto parts = split(line);
			num_variables    = std::stoi(parts[2]);
			num_clauses      = std::stoi(parts[3]);
			break;
		}

		throw std::runtime_error("Invalid line in CNF");
	}

	while (std::getline(fs, line)) {
		Clause clause;

		for (const auto& lit_str : split(line)) {
			if (lit_str == "0")
				continue;

			const auto lit = Lit::make_lit(lit_str);
			assert(lit.var() <= num_variables);
			clause.emplace_back(lit);
		}

		formula.emplace_back(std::move(clause));
	}

	assert(formula.size() == num_clauses);
	return std::make_pair(std::move(formula), num_variables);
}

Assignment parse_assignment(std::ifstream& fs, const std::size_t num_variables) {
	Assignment  assignment;
	std::string line;
	while (std::getline(fs, line)) {
		if (line.size() == 0 || line[0] == 'c')
			continue;

		if (line[0] == 's') {
			const auto parts = split(line);
			assert(parts[1] == "SATISFIABLE");
			continue;
		}

		if (line[0] == 'v') {
			const auto parts = split(line);
			assignment.reserve(parts.size() - 1);
			for (auto it = parts.begin() + 1; it != parts.end(); it++) {
				if (*it == "0")
					continue;

				const auto lit = Lit::make_lit(*it);
				assert(lit.var() <= num_variables);
				assignment.emplace_back(lit);
			}
			continue;
		}

		throw std::runtime_error("Invalid line in CNF");
	}

	return assignment;
}

int main(int argc, char* argv[]) {
	const auto start = std::chrono::high_resolution_clock::now();

	if (argc != 3)
		return EXIT_FAILURE;

	std::ifstream dimacs;
	dimacs.open(argv[1]);
	const auto formula_pair  = parse_formula(dimacs);
	const auto num_variables = std::get<1>(formula_pair);
	dimacs.close();

	std::ifstream proof;
	proof.open(argv[2]);
	const auto assignment = parse_assignment(proof, num_variables);
	proof.close();

	// Parsing is way slower than solving
	const auto done_parsing = std::chrono::high_resolution_clock::now();
	const auto sat          = check_sat(std::get<0>(formula_pair), assignment, num_variables);
	const auto done_solving = std::chrono::high_resolution_clock::now();

	if (sat)
		std::cout << "VERIFIED" << std::endl;
	else
		std::cout << "NOT VERIFIED" << std::endl;

	const auto parsing_time =
	    std::chrono::duration_cast<std::chrono::milliseconds>(done_parsing - start);
	const auto solving_time =
	    std::chrono::duration_cast<std::chrono::milliseconds>(done_solving - done_parsing);
	std::cout << "Parsing took " << parsing_time.count() << " milliseconds" << std::endl;
	std::cout << "Solving took " << solving_time.count() << " milliseconds" << std::endl;
}
