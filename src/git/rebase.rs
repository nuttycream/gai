// functions here are not so much related
// to git rebase as or relevant rebase
// mechanics as gai rebase will not
// operate similar to traditional git rebase
// in that it won't transplant commits
// to another branch, unless
// specifically specified.
// I want to avoid having any sort of
// conflict that will popup in those scenarios
// while we can check if conflictgs
// exist in the first place
//
// if that were the case, then using an --onto
// flag, and checking if conflicts exist.
// if they exist, then we bail early
// rather than leaving the
// repo in a half-rebased state
//
// gai rebase is more to "recreate" commits
// in-place, but restructed, somewhat similar
// to a git rebase -i, but less about doing
// operations (might be an option) and more
// to do with generating commits from the diff
// of the specified divergent point
