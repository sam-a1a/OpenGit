ALTER TABLE workflow_steps ADD CONSTRAINT workflow_steps_job_number_unique UNIQUE (job_id, number);
