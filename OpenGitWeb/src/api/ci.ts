import { apiClient } from "./client";

export const ciApi = {
    workflows: {
        list:    (owner: string, repo: string) =>
            apiClient.get(`/repos/${owner}/${repo}/actions/workflows`),
        get:     (owner: string, repo: string, id: string) =>
            apiClient.get(`/repos/${owner}/${repo}/actions/workflows/${id}`),
        enable:  (owner: string, repo: string, id: string) =>
            apiClient.put(`/repos/${owner}/${repo}/actions/workflows/${id}/enable`),
        disable: (owner: string, repo: string, id: string) =>
            apiClient.put(`/repos/${owner}/${repo}/actions/workflows/${id}/disable`),
        trigger: (owner: string, repo: string, id: string, ref_name: string) =>
            apiClient.post(`/repos/${owner}/${repo}/actions/workflows/${id}/dispatches`, { ref_name }),
        runs:    (owner: string, repo: string, id: string) =>
            apiClient.get(`/repos/${owner}/${repo}/actions/workflows/${id}/runs`),
    },

    runs: {
        list:   (owner: string, repo: string) =>
            apiClient.get(`/repos/${owner}/${repo}/actions/runs`),
        get:    (owner: string, repo: string, id: string) =>
            apiClient.get(`/repos/${owner}/${repo}/actions/runs/${id}`),
        cancel: (owner: string, repo: string, id: string) =>
            apiClient.post(`/repos/${owner}/${repo}/actions/runs/${id}/cancel`),
        rerun:  (owner: string, repo: string, id: string) =>
            apiClient.post(`/repos/${owner}/${repo}/actions/runs/${id}/rerun`),
        delete: (owner: string, repo: string, id: string) =>
            apiClient.delete(`/repos/${owner}/${repo}/actions/runs/${id}`),
        jobs:   (owner: string, repo: string, id: string) =>
            apiClient.get(`/repos/${owner}/${repo}/actions/runs/${id}/jobs`),
    },

    artifacts: {
        list:     (owner: string, repo: string, runId: string) =>
            apiClient.get(`/repos/${owner}/${repo}/actions/runs/${runId}/artifacts`),
        download: (owner: string, repo: string, id: string) =>
            apiClient.get(`/repos/${owner}/${repo}/actions/artifacts/${id}`),
        delete:   (owner: string, repo: string, id: string) =>
            apiClient.delete(`/repos/${owner}/${repo}/actions/artifacts/${id}`),
    },

    runners: {
        list:     () => apiClient.get("/actions/runners"),
        register: (data: object) => apiClient.post("/actions/runners", data),
        delete:   (id: string) => apiClient.delete(`/actions/runners/${id}`),
        heartbeat: (token: string, status?: string) =>
            apiClient.post("/actions/runners/heartbeat", { token, status }),
    },
};