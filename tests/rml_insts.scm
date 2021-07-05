(controller
   (perform (op print) (const "Please enter a number or 'q' for quit: "))
   (assign n (op read))
   (test (op eq?) (reg n) (const q))
   (branch (label done))
   (test (op noninteger?) (reg n))
   (branch (label controller))
   (assign continue (label fib-done))
 fib-loop
   (test (op <) (reg n) (const 2))
   (branch (label immediate-answer))
   ;; set up to compute Fib(n-1)
   (save continue)
   (assign continue (label afterfib-n-1))
   (save n)                 ; save old value of n
   (assign n (op -) (reg n) (const 1))    ; clobber n to n-1
   (goto (label fib-loop))  ; perform recursive call
 afterfib-n-1     ; upon return, val contains Fib(n-1)
   (restore n)
   (restore continue)
   ;; set up to compute Fib(n-2)
   (assign n (op -) (reg n) (const 2))
   (save continue)
   (assign continue (label afterfib-n-2))
   (save val)               ; save Fib(n-1)
   (goto (label fib-loop))
 afterfib-n-2     ; upon return, val contains Fib(n-2)
   (assign n (reg val))     ; n now contains Fib(n-2)
   (restore val)            ; val now contains Fib(n-1)
   (restore continue)
   (assign val              ; Fib(n-1) + Fib(n-2)
           (op +) (reg val) (reg n))
   (goto (reg continue))    ; return to caller, answer is in val
 immediate-answer
   (assign val (reg n))     ; base case: Fib(n) = n
   (goto (reg continue))
 fib-done
   (perform (op print-stack-statistics))
   (perform (op print) (reg val))
   (perform (op initialize-stack))
   (goto (label controller))
 done)