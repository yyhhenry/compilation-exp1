
#table(
  columns: (auto, auto),
  [描述], [特殊终结符],
  [Var], ["Var"],
  [类型关键字], [t],
  [标识符], [i],
)


#table(
  columns: (auto, auto),
  [描述], [产生式],
  [变量定义块], [$S ->$ "Var" $D | epsilon$],
  [定义列表], [$D -> D_1 D'$],
  [], [$D' -> D_1 D' | epsilon$],
  [定义], [$D_1 -> I: t;$],
  [标识符列表], [$I -> i I'$],
  [], [$I' -> , i I' | epsilon$],
)

可若干步推导出空的非终结符有：$S, D', I'$。

- $"First"(S) = {"Var", epsilon}$

- $"First"(D) = "First"(D_1) = {"i"}$

- $"First"(D') = "First"(D_1) union {epsilon} = {"i", epsilon}$

- $"First"(D_1) = "First"(I) = {i}$

- $"First"(I) = {"i"}$

- $"First"(I') = {$","$, epsilon}$

- $"Follow"(S) = {\#}$

- $"Follow"(D) = "Follow"(S) = {\#}$

- $"Follow"(D') = "Follow"(D) = {\#}$

- $"Follow"(D_1) = "First"(D') union "Follow"(D') union "Follow"(D) = {"i", \#}$

- $"Follow"(I) = {":"}$

- $"Follow"(I') = "Follow"(I) = {":"}$

- $"Select"(S -> $"Var"$D) = {$"Var"$}$

- $"Select"(S -> epsilon) = {\#}$

- $"Select"(D -> D_1 D') = {"i"}$

- $"Select"(D' -> D_1 D') = {"i"}$

- $"Select"(D' -> epsilon) = {\#}$

- $"Select"(D_1 -> I: t;) = {"i"}$

- $"Select"(I -> i I') = {"i"}$

- $"Select"(I' -> , i I') = {$","$}$

- $"Select"(I' -> epsilon) = {":"}$
