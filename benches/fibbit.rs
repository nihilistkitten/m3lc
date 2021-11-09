use criterion::{criterion_group, criterion_main, Criterion};
use m3lc::Term::*;

fn benchmark(c: &mut Criterion) {
    c.bench_function("fibbit 10", |b| {
        b.iter(|| {
            Appl {
                left: Lam {
                    param: "n".into(),
                    rule: Appl {
                        left: Lam {
                            param: "p".into(),
                            rule: Appl {
                                left: "p".into(),
                                right: Lam {
                                    param: "t".into(),
                                    rule: Lam {
                                        param: "e".into(),
                                        rule: "t".into(),
                                    }
                                    .into(),
                                }
                                .into(),
                            }
                            .into(),
                        }
                        .into(),
                        right: Appl {
                            left: Appl {
                                left: "n".into(),
                                right: Lam {
                                    param: "p".into(),
                                    rule: Appl {
                                        left: Appl {
                                            left: Lam {
                                                param: "a".into(),
                                                rule: Lam {
                                                    param: "b".into(),
                                                    rule: Lam {
                                                        param: "s".into(),
                                                        rule: Appl {
                                                            left: Appl {
                                                                left: "s".into(),
                                                                right: "a".into(),
                                                            }
                                                            .into(),
                                                            right: "b".into(),
                                                        }
                                                        .into(),
                                                    }
                                                    .into(),
                                                }
                                                .into(),
                                            }
                                            .into(),
                                            right: Appl {
                                                left: Lam {
                                                    param: "p".into(),
                                                    rule: Appl {
                                                        left: "p".into(),
                                                        right: Lam {
                                                            param: "t".into(),
                                                            rule: Lam {
                                                                param: "e".into(),
                                                                rule: "e".into(),
                                                            }
                                                            .into(),
                                                        }
                                                        .into(),
                                                    }
                                                    .into(),
                                                }
                                                .into(),
                                                right: "p".into(),
                                            }
                                            .into(),
                                        }
                                        .into(),
                                        right: Appl {
                                            left: Appl {
                                                left: Lam {
                                                    param: "m".into(),
                                                    rule: Lam {
                                                        param: "n".into(),
                                                        rule: Appl {
                                                            left: Appl {
                                                                left: "m".into(),
                                                                right: Lam {
                                                                    param: "n".into(),
                                                                    rule: Lam {
                                                                        param: "f".into(),
                                                                        rule: Lam {
                                                                            param: "x".into(),
                                                                            rule: Appl {
                                                                                left: "f".into(),
                                                                                right: Appl {
                                                                                    left: Appl {
                                                                                        left: "n"
                                                                                            .into(),
                                                                                        right: "f"
                                                                                            .into(),
                                                                                    }
                                                                                    .into(),
                                                                                    right: "x"
                                                                                        .into(),
                                                                                }
                                                                                .into(),
                                                                            }
                                                                            .into(),
                                                                        }
                                                                        .into(),
                                                                    }
                                                                    .into(),
                                                                }
                                                                .into(),
                                                            }
                                                            .into(),
                                                            right: "n".into(),
                                                        }
                                                        .into(),
                                                    }
                                                    .into(),
                                                }
                                                .into(),
                                                right: Appl {
                                                    left: Lam {
                                                        param: "p".into(),
                                                        rule: Appl {
                                                            left: "p".into(),
                                                            right: Lam {
                                                                param: "t".into(),
                                                                rule: Lam {
                                                                    param: "e".into(),
                                                                    rule: "t".into(),
                                                                }
                                                                .into(),
                                                            }
                                                            .into(),
                                                        }
                                                        .into(),
                                                    }
                                                    .into(),
                                                    right: "p".into(),
                                                }
                                                .into(),
                                            }
                                            .into(),
                                            right: Appl {
                                                left: Lam {
                                                    param: "p".into(),
                                                    rule: Appl {
                                                        left: "p".into(),
                                                        right: Lam {
                                                            param: "t".into(),
                                                            rule: Lam {
                                                                param: "e".into(),
                                                                rule: "e".into(),
                                                            }
                                                            .into(),
                                                        }
                                                        .into(),
                                                    }
                                                    .into(),
                                                }
                                                .into(),
                                                right: "p".into(),
                                            }
                                            .into(),
                                        }
                                        .into(),
                                    }
                                    .into(),
                                }
                                .into(),
                            }
                            .into(),
                            right: Appl {
                                left: Appl {
                                    left: Lam {
                                        param: "a".into(),
                                        rule: Lam {
                                            param: "b".into(),
                                            rule: Lam {
                                                param: "s".into(),
                                                rule: Appl {
                                                    left: Appl {
                                                        left: "s".into(),
                                                        right: "a".into(),
                                                    }
                                                    .into(),
                                                    right: "b".into(),
                                                }
                                                .into(),
                                            }
                                            .into(),
                                        }
                                        .into(),
                                    }
                                    .into(),
                                    right: Lam {
                                        param: "f".into(),
                                        rule: Lam {
                                            param: "x".into(),
                                            rule: "x".into(),
                                        }
                                        .into(),
                                    }
                                    .into(),
                                }
                                .into(),
                                right: Appl {
                                    left: Lam {
                                        param: "n".into(),
                                        rule: Lam {
                                            param: "f".into(),
                                            rule: Lam {
                                                param: "x".into(),
                                                rule: Appl {
                                                    left: "f".into(),
                                                    right: Appl {
                                                        left: Appl {
                                                            left: "n".into(),
                                                            right: "f".into(),
                                                        }
                                                        .into(),
                                                        right: "x".into(),
                                                    }
                                                    .into(),
                                                }
                                                .into(),
                                            }
                                            .into(),
                                        }
                                        .into(),
                                    }
                                    .into(),
                                    right: Lam {
                                        param: "f".into(),
                                        rule: Lam {
                                            param: "x".into(),
                                            rule: "x".into(),
                                        }
                                        .into(),
                                    }
                                    .into(),
                                }
                                .into(),
                            }
                            .into(),
                        }
                        .into(),
                    }
                    .into(),
                }
                .into(),
                right: Lam {
                    param: "f".into(),
                    rule: Lam {
                        param: "x".into(),
                        rule: Appl {
                            left: "f".into(),
                            right: Appl {
                                left: "f".into(),
                                right: Appl {
                                    left: "f".into(),
                                    right: Appl {
                                        left: "f".into(),
                                        right: Appl {
                                            left: "f".into(),
                                            right: Appl {
                                                left: "f".into(),
                                                right: Appl {
                                                    left: "f".into(),
                                                    right: Appl {
                                                        left: "f".into(),
                                                        right: Appl {
                                                            left: "f".into(),
                                                            right: Appl {
                                                                left: "f".into(),
                                                                right: "x".into(),
                                                            }
                                                            .into(),
                                                        }
                                                        .into(),
                                                    }
                                                    .into(),
                                                }
                                                .into(),
                                            }
                                            .into(),
                                        }
                                        .into(),
                                    }
                                    .into(),
                                }
                                .into(),
                            }
                            .into(),
                        }
                        .into(),
                    }
                    .into(),
                }
                .into(),
            }
            .reduce(false)
        })
    });
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
