class DiagnosticMock {
  message: string;
  range: unknown;
  severity: number;

  constructor(range: unknown, message: string, severity: number) {
    this.range = range;
    this.message = message;
    this.severity = severity;
  }
}

export { DiagnosticMock };
