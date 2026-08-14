export function startWorkflow(): void {}

export class WorkflowRunner {
  static constructions = 0;

  constructor() {
    WorkflowRunner.constructions += 1;
  }
}
