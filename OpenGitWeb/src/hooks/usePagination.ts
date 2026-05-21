import { useState } from "react";

export function usePagination(initialPage = 1, perPage = 30) {
    const [page, setPage]         = useState(initialPage);
    const [totalPages, setTotal]  = useState(1);

    const next = () => setPage((p) => Math.min(p + 1, totalPages));
    const prev = () => setPage((p) => Math.max(p - 1, 1));
    const go   = (p: number) => setPage(Math.max(1, Math.min(p, totalPages)));

    return { page, perPage, totalPages, setTotal, next, prev, go };
}