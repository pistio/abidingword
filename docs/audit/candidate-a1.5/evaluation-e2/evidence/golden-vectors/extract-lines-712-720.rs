                let case_only = dense(want) != dense(text)
                    && dense(want).to_lowercase() == dense(text).to_lowercase();
                let class = if want.trim() == text.trim() {
                    0
                } else if squash(want) == squash(text) || dense(want) == dense(text) || case_only {
                    1
                } else {
                    2
                };
