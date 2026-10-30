"""Deterministic variable ordering for correlation matrices."""
from __future__ import annotations

from typing import Sequence


def resolve_ordering(labels: Sequence[str], matrix: Sequence[Sequence[float | None]], strategy: str) -> list[int]:
    """Return ordered zero-based original indices.

    hclust uses average linkage over dissimilarity d(i,j)=1-r(i,j).
    Missing off-diagonal correlations are omitted from cluster-pair averages.
    Cluster-pair distances with no observed cross-pairs are unavailable.
    Equal distances are resolved by the smallest original index in each
    cluster; child clusters are oriented by that same key.
    """
    size = len(labels)
    if strategy == "original":
        return list(range(size))
    if strategy == "alphabetical":
        return sorted(range(size), key=lambda i: (str(labels[i]).casefold(), str(labels[i]), i))
    if strategy == "aoe":
        raise ValueError(
            "plot.ordering='aoe' is not supported yet; use original, alphabetical, or hclust"
        )
    if strategy != "hclust":
        raise ValueError(f"unsupported ordering strategy: {strategy!r}")
    if size <= 1:
        return list(range(size))

    distances: dict[tuple[int, int], float] = {}
    for i in range(size):
        for j in range(i + 1, size):
            value = matrix[i][j]
            if value is not None:
                distances[(i, j)] = 1.0 - float(value)

    unsupported = [i for i in range(size) if not any(i in pair for pair in distances)]
    if unsupported:
        names = ", ".join(repr(labels[i]) for i in unsupported)
        raise ValueError(
            "plot.ordering='hclust' is underdetermined for variables without "
            f"observed off-diagonal correlations: {names}; use original or alphabetical"
        )

    clusters: list[tuple[int, ...]] = [(i,) for i in range(size)]
    leaf_orders: dict[tuple[int, ...], list[int]] = {(i,): [i] for i in range(size)}
    while len(clusters) > 1:
        candidates: list[tuple[float, int, int, int, int]] = []
        for left_pos in range(len(clusters)):
            for right_pos in range(left_pos + 1, len(clusters)):
                left, right = clusters[left_pos], clusters[right_pos]
                observed = [
                    distances[(min(i, j), max(i, j))]
                    for i in left for j in right
                    if (min(i, j), max(i, j)) in distances
                ]
                if observed:
                    candidates.append((
                        sum(observed) / len(observed),
                        min(left), min(right), left_pos, right_pos,
                    ))
        if not candidates:
            remaining = [[labels[i] for i in cluster] for cluster in clusters]
            raise ValueError(
                "plot.ordering='hclust' cannot connect the remaining clusters "
                f"because all cross-cluster correlations are missing: {remaining!r}"
            )
        _, _, _, left_pos, right_pos = min(candidates)
        left, right = clusters[left_pos], clusters[right_pos]
        left_order, right_order = leaf_orders[left], leaf_orders[right]
        if min(left) > min(right):
            left, right = right, left
            left_order, right_order = right_order, left_order
        merged = tuple(sorted(left + right))
        leaf_orders[merged] = left_order + right_order
        clusters = [
            cluster for pos, cluster in enumerate(clusters)
            if pos not in (left_pos, right_pos)
        ]
        clusters.append(merged)
        clusters.sort(key=min)

    return leaf_orders[clusters[0]]
