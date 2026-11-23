## 1. A term of the whole system

- [ ] 1.1 Let a term bind one repository or the whole system in `Proposal::check`, with no repository in the fingerprint of a term of the whole system. `boom:test` proves the scenarios "A term proposed", "A term of the whole system", "A term for two repositories", "One word in two repositories" and "A term with no replaces"
- [ ] 1.2 Refuse `boom propose term` with no `--repository` and no `--system` in a folder of no repository, and say a term binds one repository or the whole system. `boom:test` proves the scenario "A term with no repository"
- [ ] 1.3 Keep the binding of a term's base in an amendment, and refuse an amendment across bindings. `boom:test` proves the scenarios "An amendment of a term of the whole system" and "An amendment across bindings"
- [ ] 1.4 `boom:test` proves the scenarios "A name taken by another kind", "A name whose claim stands", "Two first versions waiting", "A term with the name of an invariant", "A term of the whole system with the name of an invariant" and "A word of the whole system and of a repository"
- [ ] 1.5 `boom:test` proves the scenarios "A word with a term in two repositories", "One repository chosen" and "A word of the whole system and of a repository shown"

## 2. The term that stands for a repository

- [ ] 2.1 Drop the term of the whole system from `State::standing_for` for a repository whose own term with that word stands. `boom:test` proves the scenarios "The term of the whole system in a repository" and "A repository with its own term"
- [ ] 2.2 `boom:test` proves the scenario "Searched in a repository and in all"

## 3. The check

- [ ] 3.1 Read the terms of the whole system with those of the repository of each file in `terms::accepted`, less each one whose word the repository's own term has. Name "the whole system" in words and `repository` `null` in JSON. `boom:test` proves the scenarios "A term of the whole system in each repository", "A term of the whole system, as JSON" and "A repository with its own term for the word"
- [ ] 3.2 `boom:test` proves the scenarios "An old word found", "No old word", "Answered as JSON", "Nothing found, as JSON", "A term of another repository" and "A word that no term replaces"

## 4. The pages and the gate

- [ ] 4.1 Write a term of the whole system into `README.md` and the help of `boom propose`, in Plain
- [ ] 4.2 Run `boom:format`, `boom:lint`, `boom:typecheck`, `boom:test`, `root:specs`, `root:changes` and `root:terms` through `sluice exec main`
