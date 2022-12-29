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

// Thoughts: DIMACS parsing takes way longer than proof checking. It also uses
// a ton of memory. We could stream the dimacs in, and check each clause as it
// comes in, preventing the need to store the whole formula in memory. This is
// a nice optimisation as it reduces the amount of memory needed and means in
// the case of early exit you dont have to parse everything, but is potentially
// harder to formalise and in many ways unecessary because of how little time
// proof checking takes

enum class TriBool : uint8_t {
	None  = 0x00,
	False = 0x01,
	True  = 0x02,
};

using Var           = uint32_t;
using AssignmentVec = std::vector<TriBool>;

class Lit {
public:
	constexpr Lit(const int32_t dimacs_lit) : lit(0) {
		assert(dimacs_lit != 0);
		const bool is_pos = dimacs_lit > 0;
		const Var  var    = is_pos ? dimacs_lit : -dimacs_lit;
		lit               = var | (is_pos ? pos_mask : 0);
	}

	constexpr Lit(const Var var, const bool is_pos) : lit(var | (is_pos ? pos_mask : 0)) {
		assert(var != 0);
	}

	constexpr Var     raw() const { return lit; }
	constexpr Var     var() const { return lit & ~pos_mask; }
	constexpr bool    is_pos() const { return lit & pos_mask; }
	constexpr TriBool tri_bool() const { return is_pos() ? TriBool::True : TriBool::False; }

	bool sat_by(const AssignmentVec& assignment) const { return assignment[var()] == tri_bool(); }

	bool operator==(const Lit& rhs) const { return lit == rhs.lit; }
	bool operator<(const Lit& rhs) const { return var() < rhs.var(); }

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
using ClauseIdx  = std::size_t;
using Formula    = std::vector<Clause>;
using Assignment = std::vector<Lit>;

inline std::vector<std::string> split(std::string const& input) {
	std::istringstream       buffer(input);
	std::vector<std::string> ret{std::istream_iterator<std::string>(buffer), {}};
	return ret;
}

inline std::tuple<Formula, std::size_t> parse_formula(std::ifstream& fs) {
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

	formula.reserve(num_clauses);

	int    lit_int;
	Clause clause;
	while (fs >> lit_int) {
		if (lit_int == 0) {
			formula.emplace_back(std::move(clause));
			clause.clear();
			continue;
		}

		Lit lit{lit_int};
		assert(lit.var() <= num_variables);
		clause.emplace_back(lit);
	}

	if (!clause.empty())
		formula.emplace_back(std::move(clause));

	assert(formula.size() == num_clauses);
	return std::make_pair(std::move(formula), num_variables);
}

static AssignmentVec to_assignment_vec(const Assignment& assignment, const std::size_t num_vars) {
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

static bool is_sat(const Formula& formula, const AssignmentVec& assignment) {
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

static bool check_sat(
    const Formula& formula, const Assignment& assignment, const std::size_t num_vars
) {
	return is_sat(formula, to_assignment_vec(assignment, num_vars));
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

	const auto done_dimacs = std::chrono::high_resolution_clock::now();

	std::ifstream proof;
	proof.open(argv[2]);
	const auto assignment = parse_assignment(proof, num_variables);
	proof.close();

	const auto done_proof = std::chrono::high_resolution_clock::now();

	// Parsing is way slower than solving
	const auto sat          = check_sat(std::get<0>(formula_pair), assignment, num_variables);
	const auto done_solving = std::chrono::high_resolution_clock::now();

	if (sat)
		std::cout << "VERIFIED" << std::endl;
	else
		std::cout << "NOT VERIFIED" << std::endl;

	const auto dimacs_time =
	    std::chrono::duration_cast<std::chrono::milliseconds>(done_dimacs - start);
	const auto proof_parse_time =
	    std::chrono::duration_cast<std::chrono::milliseconds>(done_proof - done_dimacs);
	const auto solving_time =
	    std::chrono::duration_cast<std::chrono::milliseconds>(done_solving - done_proof);
	std::cout << "DIMACS Parsing took " << dimacs_time.count() << " milliseconds" << std::endl;
	std::cout << "Proof Parsing took " << proof_parse_time.count() << " milliseconds" << std::endl;
	std::cout << "Solving took " << solving_time.count() << " milliseconds" << std::endl;
}
